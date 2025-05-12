use std::collections::HashMap;
use database::core::{is_table_exists, DBConn};
use crate::handler::build_rpc_value;

/// 处理HEAD请求的异步方法，主要用于检查表是否存在和记录计数
/// 
/// # 参数
/// * `body_map` - 包含请求参数的HashMap，键为表名(String)，值为查询条件(serde_json::Value)
/// 
/// # 返回值
/// 返回serde_json::Value类型的JSON响应数据，包含操作结果
/// 
/// # 错误处理
/// - 如果表不存在，返回错误信息
/// - 如果参数格式错误，返回错误信息
/// - 如果查询失败，返回错误信息
pub async fn handle_head(db: &DBConn, body_map: HashMap<String, serde_json::Value>) -> serde_json::Value {
    let mut result_map = HashMap::new();
    let mut code = 200;

    for (table_key, param) in body_map {
        match param.as_object() {
            Some(param_map) => {
                // 解析 schema & table
                let schema: &str;
                let table: &str;
                let schema_table_vec = table_key.split('.').collect::<Vec<&str>>();
                if schema_table_vec.len() == 2 {
                    schema = schema_table_vec[0];
                    table = schema_table_vec[1];
                } else {
                    code = 400;
                    result_map.insert((&table_key).to_string(), serde_json::json!(format!("{}'s database should be specified", &table_key)));
                    continue;
                }
                // 检查表是否存在，不存在则记录错误
                if !is_table_exists(&schema, &table) {
                    code = 400;
                    result_map.insert((&table_key).to_string(), serde_json::json!(format!("table {} not exists", &table_key)));
                    continue;
                }

                // 统计计数
                match count_one(db, &schema, &table, param_map).await {
                    Ok(id) => {
                        result_map.insert(table_key.clone(), serde_json::json!(id));
                    },
                    Err(err) => {
                        code = 400;
                        result_map.insert(table_key.clone(), serde_json::Value::String(err));
                    }
                }
            },
            None => {
                code = 400;
                result_map.insert(table_key.clone(), serde_json::Value::String(format!("参数格式错误，value: {:?}", param)));
            }
        }
    }
    build_rpc_value(code, None, result_map)
}

async fn count_one(db: &DBConn, schema: &str, table: &str, kvs: &serde_json::Map<String, serde_json::Value>) -> Result<i64, String> {
    let mut where_causes = Vec::new();
    let mut values = Vec::new();
    for (field, value) in kvs {
        where_causes.push(format!("{}=?", field.as_str()));
        values.push(value.to_string());
    }
    let sql = format!("SELECT count(1) FROM {}.{} WHERE {};", schema, table, where_causes.join(" AND "));
    match db.count(&sql, values).await {
        Ok(cnt) => { Ok(cnt) },
        Err(e) => Err(e.to_string())
    }
}