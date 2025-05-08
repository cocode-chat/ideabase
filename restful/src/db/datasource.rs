use lazy_static::lazy_static;
use std::sync::RwLock;
use std::collections::HashMap;
use sqlx::{mysql::{MySqlColumn, MySqlRow, MySqlPool}, Column, Row, TypeInfo, types::Decimal};
use common::utils::base64_encode;
use common::yaml::DataSource;
use crate::db::{ColumnMeta, DbMeta, TableMeta};

lazy_static! {
    static ref DB_MAP: RwLock<HashMap<String, DbMeta>> = RwLock::new(HashMap::new());
    static ref DB_TABLES_MAP: RwLock<HashMap<String, Vec<String>>> = RwLock::new(HashMap::new());
    static ref TABLE_MAP: RwLock<HashMap<String, TableMeta>> = RwLock::new(HashMap::new());
}
// MySQL系统数据库列表
const SYS_DB: &[&str] = &["information_schema", "mysql", "performance_schema", "sys"];

#[derive(Debug, Clone)]
pub struct DBConn {
    pool: MySqlPool,
}

impl DBConn {
    pub async fn new(datasource: DataSource) -> Self {
        let host = datasource.host;
        let port = datasource.port;
        let username = datasource.username;
        let password = datasource.password;
        let connection_string = format!("mysql://{}:{}@{}:{}?charset=utf8mb4", username, password, host, port);
        let pool = MySqlPool::connect(&connection_string).await.expect("mysql connection error");
        let mut ds: DBConn = DBConn { pool };
        // 初始化链接池
        ds.init().await.expect("mysql datasource init error");
        ds
    }

    async fn init(&mut self) -> Result<(), sqlx::Error> {
        self.load_db().await.expect("mysql database load error");
        for db_name in DB_MAP.read().unwrap().keys() {
            self.load_db_table(db_name).await.expect("mysql schema table load error");
        }
        Ok(())
    }

    async fn load_db(&mut self)-> Result<(), sqlx::Error> {
        let list_db_sql = "SELECT table_schema AS name, ROUND(SUM(data_length + index_length) / 1024 / 1024, 2) AS size FROM information_schema.tables GROUP BY table_schema;";
        let db_list = sqlx::query(&list_db_sql).fetch_all(&self.pool).await?;
        let mut all_dbs = DB_MAP.write().unwrap();
        for db_row in db_list.iter() {
            let db_name: String = db_row.get("name");
            let db_size: Decimal = db_row.get("size");
            if SYS_DB.contains(&db_name.as_str()) { continue; }
            all_dbs.insert(db_name.clone(), DbMeta { name: db_name, size: db_size.to_string().parse::<f64>().unwrap_or(0.0) });
        }
        Ok(())
    }

    async fn load_db_table(&mut self, schema: &str) -> Result<(), sqlx::Error> {
        let list_db_table_sql = format!("select TABLE_NAME, TABLE_COMMENT from information_schema.tables where table_schema='{}' and table_type='BASE TABLE'", schema);
        let tables = sqlx::query(&list_db_table_sql).fetch_all(&self.pool).await?;
        let mut db_tables = DB_TABLES_MAP.write().unwrap();
        let mut table_name_list = Vec::<String>::new();
        let mut all_tables = TABLE_MAP.write().unwrap();
        for table_row in tables {
            let table_name: String = table_row.get("TABLE_NAME");
            let table_comment: String = table_row.get("TABLE_COMMENT");
            let table_name = table_name.to_lowercase();
            let columns = self.load_table_meta(&schema, &table_name).await?;
            let table_meta = TableMeta { schema: schema.to_string(), name: table_name.clone(), columns, comment: Some(table_comment) };
            let table_key = format!("{}.{}", schema, &table_name);
            log::info!( "mysql.table: {} loaded", &table_key);
            all_tables.insert(table_key, table_meta);
            table_name_list.push(table_name.clone());
        }
        db_tables.insert(schema.to_string(), table_name_list);

        Ok(())
    }

    async fn load_table_meta(&self, schema: &str, table_name: &str) -> Result<HashMap<String, ColumnMeta>, sqlx::Error> {
        let columns: Vec<ColumnMeta> = sqlx::query_as(&format!("SHOW FULL COLUMNS FROM {}.{}", schema, table_name))
            .fetch_all(&self.pool).await?;
        let mut column_map = HashMap::new();
        for column in columns {
            column_map.insert(column.field.clone(), column);
        }
        Ok(column_map)
    }



    pub async fn query_one(&self, sql: &str, params: Vec<String>) -> Result<Option<HashMap<String, serde_json::Value>>, sqlx::Error> {
        let sql = if !sql.to_lowercase().contains("limit") {
            format!("{} limit 1", sql)
        } else {
            sql.to_string()
        };

        let mut query = sqlx::query(&sql);
        for param in params {
            query = query.bind(param);
        }
        let row = query.fetch_optional(&self.pool).await?;

        if let Some(row) = row {
            let mut record = HashMap::new();
            let columns = row.columns();
            for column in columns {
                let value: serde_json::Value = Self::get_column_val(&row, column);
                record.insert(column.name().to_string(), value);
            }
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }
    pub async fn query_list(&self, sql: &str, params: Vec<String>) -> Result<Vec<HashMap<String, serde_json::Value>>, sqlx::Error> {
        let mut query = sqlx::query(&sql);
        for param in params {
            query = query.bind(param);
        }
        let rows = query.fetch_all(&self.pool).await?;

        let mut results = Vec::new();
        for row in rows {
            let mut record = HashMap::new();
            let columns = row.columns();
            for column in columns {
                let value: serde_json::Value = Self::get_column_val(&row, column);
                record.insert(column.name().to_string(), value);
            }
            results.push(record);
        }

        Ok(results)
    }

    fn get_column_val(row: &MySqlRow, column: &MySqlColumn) -> serde_json::Value {
        match column.type_info().name() {
            "BIGINT" | "INT" => {
                let v: Option<i64> = row.try_get(column.name()).ok();
                v.map(serde_json::Value::from).unwrap_or(serde_json::Value::Null)
            }
            "DATETIME" | "TIMESTAMP" | "DATE" | "TIME" => {
                let v: Option<chrono::NaiveDateTime> = row.try_get(column.name()).ok();
                v.map(|val| serde_json::Value::String(val.to_string())).unwrap_or(serde_json::Value::Null)
            }
            "TINYTEXT" | "TEXT" | "MEDIUMTEXT" | "LONGTEXT" => {
                let v: Option<String> = row.try_get(column.name()).ok();
                v.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null)
            }
            "JSON" => {
                let v: Option<serde_json::Value> = row.try_get(column.name()).ok();
                v.unwrap_or(serde_json::Value::Null)
            }
            "BLOB" | "TINYBLOB" | "MEDIUMBLOB" | "LONGBLOB" | "VARBINARY" | "BINARY" => {
                let v: Option<Vec<u8>> = row.try_get(column.name()).ok();
                v.map(|bytes| {
                    match String::from_utf8(bytes.clone()) {
                        Ok(s) => serde_json::Value::String(s),
                        Err(_) => serde_json::Value::String(base64_encode(bytes))
                    }
                }).unwrap_or(serde_json::Value::Null)
            }
            _ => {
                let v: Option<String> = row.try_get(column.name()).ok();
                v.map(serde_json::Value::from).unwrap_or(serde_json::Value::Null)
            }
        }
    }

    pub async fn insert(&self, sql: &str) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(sql)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() as i64)
    }

    pub async fn update(&self, sql: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(sql)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn delete(&self, sql: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(sql)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn count(&self, sql: &str, params: Vec<String>) -> Result<i64, sqlx::Error> {
        let mut query = sqlx::query(&sql);
        for param in params {
            query = query.bind(param);
        }
        let count_result = query.fetch_one(&self.pool).await?;
        let count = count_result.get(0);
        Ok(count)
    }

    pub async fn create_table(&self, sql: &str) -> Result<(), sqlx::Error> {
        let exec_result = sqlx::query(sql)
            .execute(&self.pool)
            .await;
        match exec_result {
            Ok(_) => Ok(()),
            Err(err) => {
                Err(sqlx::Error::from(err))
            }
        }
    }
}

pub fn is_table_exists(schema: &str, table: &str) -> bool {
    let table_key = format!("{}.{}", schema, table);
    let all_tables = TABLE_MAP.read().unwrap();
    all_tables.contains_key(table_key.as_str())
}

pub fn get_table(schema: &str, table: &str) -> Option<TableMeta> {
    let all_tables = TABLE_MAP.read().unwrap();
    let table_key = format!("{}.{}", schema, table);
    let table_meta_opt = all_tables.get(table_key.as_str());
    match table_meta_opt {
        None => { None }
        Some(table_meta) => { Some(table_meta.clone()) }
    }
}

pub fn get_table_name_list(schema: &str) -> HashMap<String, String> {
    let mut table_name_map = HashMap::new();
    let all_tables = TABLE_MAP.read().unwrap();
    let db_tables = DB_TABLES_MAP.read().unwrap();
    let tables = db_tables.get(schema).unwrap();
    for table_name in tables.iter() {
        let table_opt = all_tables.get(table_name.as_str());
        match table_opt {
            None => {}
            Some(table) => {
                let comment_opt = table.comment.clone();
                let comment = if let Some(comment) = comment_opt { comment } else { "".to_string() };
                table_name_map.insert(table_name.clone(), comment);
            }
        }
    }
    table_name_map
}
