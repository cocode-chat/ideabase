use std::collections::HashMap;
use serde_json::Value;
use langchain_rust::{
    add_documents,
    schemas::{Document as LangChainDocument},
    text_splitter::{TokenSplitter, TextSplitter}
};

use database::core::DBConn;
use crate::core::vector::get_vector_store;

const PAGE_SIZE: i64 = 1000;

/// 向向量数据库集合中添加文档
/// 
/// # 参数
/// * `db_conn`: 数据库连接对象
/// * `collection_name`: 目标集合名称
/// * `source_type`: 数据源类型标识
/// * `data_config`: JSON格式的配置数据，包含数据库表信息、字段映射等
pub async fn init_collection_documents(db_conn: &DBConn, collection_name: &str, source_type: &str, vector_config: Value) {
    log::info!("Starting init vector db, source_type: {} {}", collection_name, source_type);
    // 1. 解析主查询参数
    let (database, table, columns, metadata_kv) = match parse_main_query_params(&vector_config) {
        Ok(res) => res,
        Err(msg) => {
            log::error!("parse_main_query_params: {} {}", msg, vector_config);
            return;
        }
    };
    // 2. 解析列配置，分离普通列和子查询列
    let (normal_columns, sub_query_map) = parse_columns(&columns, &vector_config);
    // 3. 解析子查询信息
    let sub_query_info = parse_sub_queries(&sub_query_map);

    // 4. 获取主表记录总数
    let main_count_sql = format!("SELECT COUNT(*) FROM {database}.{table}");
    let count = match db_conn.count(&main_count_sql, vec![]).await {
        Ok(cnt) => {
            log::info!("collection_name: {}, main_data.count: {}", collection_name, cnt);
            cnt
        },
        Err(err) => {
            log::error!("Failed to count records in table {database}.{table}: {}", err);
            return;
        }
    };

    // 5. 计算分页循环次数
    let loop_cnt = (count + PAGE_SIZE - 1) / PAGE_SIZE;
    for page in 0..loop_cnt {
        let offset = page * PAGE_SIZE;
        let columns_str = normal_columns.join(", ");
        // 6. 构建主表分页查询SQL
        let main_page_sql = format!("SELECT {columns_str} FROM {database}.`{table}` LIMIT {PAGE_SIZE} OFFSET {offset};");
        // 7. 获取主表分页数据
        let (src_document_list, main_related_kvs) =
            match fetch_main_table_data(db_conn, &main_page_sql, source_type, &metadata_kv, &sub_query_info.main_related_fields).await {
                Ok(records) => records,
                Err(err) => {
                    log::error!("vector_data_etl.fetch_main_table_data {} {} {}", collection_name, main_page_sql, err);
                    return;
                }
            };

        // 8. 获取子查询数据
        let sub_query_placeholder_value_map = fetch_sub_query_data(db_conn, &sub_query_info, &main_related_kvs, collection_name).await;
        // 9. 组装最终文档
        let page_chunked_documents = assemble_documents(src_document_list, &sub_query_info, &sub_query_placeholder_value_map).await;
        // 10. 文档写入向量数据库
        if let Some(vector_store) = get_vector_store(collection_name) {
            let result = add_documents!(vector_store, &page_chunked_documents).await.map_err(|e| {
                log::error!("Error adding chunked_item_docs: {:?}", e);
            });
            match result {
                Ok(records) => {
                    log::info!("collection_name: {}, add_documents.len: {}", collection_name, records.len());
                }
                Err(err) => {
                    log::error!("Error adding documents: {:?}", err);
                }
            }
        } else {
            log::error!("Collection Vector Store not found: {}", collection_name);
        }
    }
}

/// 解析主查询参数配置
///
/// # 参数
/// * `data_config`: JSON格式的配置数据，包含数据库连接信息、表结构和元数据映射
///
/// # 返回值
/// 返回Result类型，成功时包含元组：
/// - String: 数据库名称
/// - String: 表名称
/// - String: 列字段列表(逗号分隔)
/// - HashMap<String, String>: 元数据字段映射关系
/// 失败时返回错误信息字符串
fn parse_main_query_params(data_config: &Value) -> Result<(String, String, String, HashMap<String, String>), String> {
    // 1. 从配置中获取数据库名称
    let database = data_config
        .get("database")
        .and_then(|v| v.as_str())  // 确保值为字符串类型
        .ok_or_else(|| "Missing 'database' in data_config".to_string())?  // 不存在时返回错误
        .to_string();  // 转换为String类型

    // 2. 从配置中获取表名称
    let table = data_config
        .get("table")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'table' in data_config".to_string())?
        .to_string();

    // 3. 从配置中获取列字段列表
    let columns = data_config
        .get("column")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'column' in data_config".to_string())?
        .to_string();

    // 4. 从配置中获取元数据映射对象
    let metadata_obj = data_config
        .get("metadata")
        .and_then(|v| v.as_object())  // 确保值为JSON对象
        .ok_or_else(|| "Missing 'metadata' in data_config".to_string())?;

    // 5. 构建元数据键值对映射
    let metadata_kv: HashMap<String, String> = metadata_obj
        .iter()
        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))  // 过滤并转换键值对
        .collect();

    Ok((database, table, columns, metadata_kv))
}

/// 解析列字段配置，分离普通列和子查询列
///
/// # 参数
/// * `columns`: 逗号分隔的列名字符串
/// * `data_config`: JSON格式的配置数据，包含子查询定义
///
/// # 返回值
/// 返回元组包含：
/// - Vec<String>: 普通列名列表
/// - HashMap<String, Value>: 子查询映射表(键为子查询字段名，值为子查询配置)
fn parse_columns(columns: &str, data_config: &Value) -> (Vec<String>, HashMap<String, Value>) {
    // 初始化普通列和子查询列的存储容器
    let mut normal_columns = Vec::new();
    let mut sub_query_map = HashMap::<String, Value>::new();

    // 遍历每个列定义(按逗号分割并去除前后空格)
    for col in columns.split(',').map(|s| s.trim()) {
        if col.starts_with('@') {
            // 处理子查询列(以@开头的列名)
            if let Some(sub_query) = data_config.get(col) {
                // 将子查询配置存入映射表
                sub_query_map.insert(col.to_string(), sub_query.clone());
            }
        } else if !col.is_empty() {
            // 处理普通列(非空列名)
            normal_columns.push(col.to_string());
        }
    }
    
    // 返回分离后的普通列和子查询映射
    (normal_columns, sub_query_map)
}

// 子查询相关信息结构体
struct SubQueryInfo {
    main_related_fields: Vec<String>,
    sub_query_title_map: HashMap<String, String>,
    sub_query_sql_map: HashMap<String, String>,
    sub_query_placeholder_map: HashMap<String, HashMap<String, String>>,
}

/// 解析子查询配置信息
///
/// # 参数
/// * `sub_query_map`: 子查询映射表，键为子查询字段名(@开头)，值为子查询配置(JSON格式)
///
/// # 返回值
/// 返回SubQueryInfo结构体，包含：
/// - main_related_fields: 主表关联字段列表
/// - sub_query_title_map: 子查询标题映射
/// - sub_query_sql_map: 子查询SQL语句映射  
/// - sub_query_placeholder_map: 子查询占位符映射
fn parse_sub_queries(sub_query_map: &HashMap<String, Value>) -> SubQueryInfo {
    // 初始化存储容器
    let mut main_related_fields = Vec::<String>::new();  // 主表关联字段
    let mut sub_query_title_map = HashMap::<String, String>::new();  // 子查询标题映射
    let mut sub_query_sql_map = HashMap::<String, String>::new();  // 子查询SQL映射
    let mut sub_query_placeholder_map = HashMap::<String, HashMap<String, String>>::new();  // 占位符映射

    // 遍历每个子查询配置
    for (sub_field, sub_query) in sub_query_map {
        // 1. 提取子查询标题和SQL语句
        if let (Some(sub_title), Some(sub_sql)) = (
            sub_query.get("title").and_then(|v| v.as_str()),  // 获取子查询标题
            sub_query.get("sql").and_then(|v| v.as_str())    // 获取子查询SQL
        ) {
            // 2. 解析SQL中的占位符(?开头的参数)
            let mut sub_query_placeholder_kv = HashMap::<String, String>::new();
            let re = regex::Regex::new(r"\?(\w+)").unwrap();  // 匹配?param格式的占位符
            
            // 3. 提取所有占位符字段
            for cap in re.captures_iter(sub_sql) {
                let placeholder_field = cap.get(1).unwrap().as_str();  // 获取占位符字段名
                let placeholder = format!("?{}", placeholder_field);  // 构造完整占位符
                
                // 4. 存储占位符映射关系
                sub_query_placeholder_kv.insert(placeholder, placeholder_field.to_string());
                // 记录主表关联字段
                main_related_fields.push(placeholder_field.to_string());
            }

            // 5. 存储子查询信息
            sub_query_placeholder_map.insert(sub_field.clone(), sub_query_placeholder_kv);
            sub_query_title_map.insert(sub_field.clone(), sub_title.to_string());
            sub_query_sql_map.insert(sub_field.clone(), sub_sql.to_string());
        }
    }

    // 返回解析结果
    SubQueryInfo {
        main_related_fields,
        sub_query_title_map,
        sub_query_sql_map,
        sub_query_placeholder_map,
    }
}

#[derive(Debug, Clone)]
struct DocData {
    pub record: HashMap<String, Value>,
    pub metadata: HashMap<String, Value>,
}

/// 查询主表数据并构建文档结构
///
/// # 参数
/// * `db_conn`: 数据库连接对象
/// * `main_page_sql`: 主表分页查询SQL语句
/// * `source_type`: 数据源类型标识
/// * `metadata_kv`: 元数据字段映射关系
/// * `main_related_fields`: 主表关联字段列表(用于子查询)
///
/// # 返回值
/// 返回Result类型，成功时包含元组：
/// - Vec<DocData>: 文档数据列表
/// - HashMap<String, Vec<Value>>: 关联字段值映射(用于后续子查询)
/// 失败时返回错误信息字符串
async fn fetch_main_table_data(db_conn: &DBConn, main_page_sql: &str, source_type: &str, metadata_kv: &HashMap<String, String>, main_related_fields: &Vec<String>) -> Result<(Vec<DocData>, HashMap<String, Vec<Value>>), String> {
    // 初始化返回数据结构
    let mut src_document_list = Vec::<DocData>::new();  // 文档数据列表
    let mut main_related_kvs = HashMap::<String, Vec<Value>>::new();  // 关联字段值映射

    // 执行主表查询
    match db_conn.query_list(main_page_sql, vec![]).await {
        Err(err) => Err(format!("Failed to query records in table {}: {}", main_page_sql, err)),
        Ok(records) => {
            // 处理每条查询记录
            for record in records {
                let mut metadata = HashMap::<String, Value>::new();
                
                // 1. 添加数据源类型元数据
                metadata.insert("src_type".to_string(), Value::String(source_type.to_string()));
                
                // 2. 处理配置的元数据字段映射
                for (sub_field, main_field) in metadata_kv {
                    if let Some(field_val) = record.get(sub_field) {
                        metadata.insert(main_field.to_string(), field_val.clone());
                    }
                }
                
                // 3. 收集关联字段值(用于后续子查询)
                for main_field in main_related_fields {
                    if let Some(field_val) = record.get(main_field) {
                        main_related_kvs
                            .entry(main_field.to_string())
                            .or_insert(Vec::new())
                            .push(field_val.clone());
                    }
                }
                
                // 4. 构建文档数据对象
                src_document_list.push(DocData { record, metadata });
            }
            
            // 返回处理结果
            Ok((src_document_list, main_related_kvs))
        }
    }
}

/// 查询子表数据并构建关联数据映射
///
/// # 参数
/// * `db_conn`: 数据库连接对象
/// * `sub_query_info`: 子查询配置信息
/// * `main_related_kvs`: 主表关联字段值映射(来自主表查询结果)
///
/// # 返回值
/// 返回HashMap类型，键为子查询键(格式: "子查询字段/关联字段/关联值")，
/// 值为子查询结果记录列表
async fn fetch_sub_query_data(db_conn: &DBConn, sub_query_info: &SubQueryInfo, main_related_kvs: &HashMap<String, Vec<Value>>, collection_name: &str) -> HashMap<String, Vec<HashMap<String, Value>>> {
    // 初始化子查询结果映射表
    let mut sub_query_placeholder_value_map = HashMap::<String, Vec<HashMap<String, Value>>>::new();

    // 遍历每个子查询SQL配置
    for (main_field_placeholder, sub_sql) in &sub_query_info.sub_query_sql_map {
        // 1. 替换SQL中的占位符为实际值
        let sub_query_sql = replace_subquery_sql_placeholders(main_field_placeholder, sub_sql, &sub_query_info.sub_query_placeholder_map, main_related_kvs);

        log::info!("collection_name: {}, sub_query_sql: {}", collection_name, &sub_query_sql);
        // 2. 执行子查询
        if let Ok(records) = db_conn.query_list(&sub_query_sql, vec![]).await {
            // 处理每条子查询记录
            for record in records {
                // 3. 遍历关联字段，构建子查询键
                for main_field in &sub_query_info.main_related_fields {
                    if let Some(main_field_value) = record.get(main_field) {
                        // 构建子查询键(格式: "子查询字段/关联字段/关联值")
                        let sub_query_key = format!("{main_field_placeholder}/{main_field}/{main_field_value}");
                        
                        // 4. 将子查询结果存入映射表
                        sub_query_placeholder_value_map
                            .entry(sub_query_key)
                            .or_insert(Vec::new())
                            .push(record.clone());
                    }
                }
            }
        }
    }

    // 返回子查询结果映射表
    sub_query_placeholder_value_map
}

/// 组装最终文档结构，合并主表数据和子查询数据
///
/// # 参数
/// * `src_document_list`: 主表文档数据列表
/// * `sub_query_info`: 子查询配置信息
/// * `sub_query_placeholder_value_map`: 子查询结果映射表
///
/// # 返回值
/// 返回LangChainDocument列表，包含完整文档内容和元数据
async fn assemble_documents(mut src_document_list: Vec<DocData>, sub_query_info: &SubQueryInfo, sub_query_placeholder_value_map: &HashMap<String, Vec<HashMap<String, Value>>>) -> Vec<LangChainDocument> {
    let mut page_documents = Vec::<LangChainDocument>::new();
    
    // 遍历每个主表文档
    for document in &mut src_document_list {
        let record = &document.record;
        let metadata = &document.metadata;

        let mut sub_content_list = Vec::new();
        
        // 1. 处理每个子查询字段
        for (main_field_placeholder, kvs) in &sub_query_info.sub_query_placeholder_map {
            // 获取子查询标题
            let sub_title = sub_query_info
                .sub_query_title_map
                .get(main_field_placeholder)
                .unwrap();
            
            // 2. 处理子查询关联字段
            for (_, main_field) in kvs {
                if let Some(main_field_value) = record.get(main_field) {
                    // 构建子查询键
                    let sub_query_key =
                        format!("{main_field_placeholder}/{main_field}/{main_field_value}");
                    
                    // 3. 获取关联的子查询结果
                    if let Some(sub_records) = sub_query_placeholder_value_map.get(&sub_query_key) {
                        let mut sub_record_content_list = Vec::new();
                        
                        // 4. 格式化每个子查询记录
                        for sub_record in sub_records {
                            let sub_content = format_content(&sub_record, &vec![], false);
                            sub_record_content_list.push(format!(" - {sub_content}"));
                        }
                        
                        // 5. 组合子查询内容
                        let sub_content = format!(
                            "{sub_title}: \n{sub_content_list}",
                            sub_content_list = sub_record_content_list.join("\n")
                        );
                        sub_content_list.push(sub_content);
                    }
                }
            }
        }
        
        // 6. 格式化主表内容
        let content = format_content(&record, &vec![], true);
        
        // 7. 合并主表和子查询内容
        let doc_content = format!("{content}{}", sub_content_list.join("\n"));
        
        // 8. 构建最终文档对象
        let vec_doc = LangChainDocument::new(doc_content).with_metadata(metadata.clone());
        page_documents.push(vec_doc);
    }

    // 使用 TokenSplitter 对文档进行分块
    let chunked_documents = TokenSplitter::default().split_documents(&page_documents).await;
    match chunked_documents {
        Ok(documents) => { documents }
        Err(err) => {
            log::error!("Error splitting documents: {}", err);
            vec![]
        }
    }
}


/// 替换子查询SQL中的占位符为实际值
///
/// # 参数
/// * `main_field`: 主查询字段名(带@前缀)
/// * `sub_sql`: 子查询SQL语句(包含占位符)
/// * `sub_query_placeholder_map`: 子查询占位符映射表
/// * `main_related_kvs`: 主表关联字段值映射
///
/// # 返回值
/// 返回替换后的SQL字符串
fn replace_subquery_sql_placeholders(main_field: &str, sub_sql: &str, sub_query_placeholder_map: &HashMap<String, HashMap<String, String>>, main_related_kvs: &HashMap<String, Vec<Value>>) -> String {
    // 1. 创建SQL副本用于替换操作
    let mut replaced_sql = sub_sql.to_owned();
    
    // 2. 获取当前主字段对应的占位符映射
    if let Some(placeholder_map) = sub_query_placeholder_map.get(main_field) {
        // 3. 遍历每个占位符及其对应的主表字段
        for (placeholder, main_field) in placeholder_map {
            // 4. 获取主表字段对应的值列表
            if let Some(values) = main_related_kvs.get(main_field) {
                // 5. 将值列表转换为逗号分隔的字符串
                let value_str = values.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                
                // 6. 替换SQL中的占位符为实际值
                replaced_sql = replaced_sql.replace(&placeholder.to_string(), &value_str);
            } else {
                // 7. 记录警告日志(未找到关联字段值)
                log::warn!("main_related_kvs 未找到字段: {}", main_field);
            }
        }
    }
    
    // 8. 返回替换后的SQL
    replaced_sql.to_string()
}

/// 格式化记录内容为字符串
///
/// # 参数
/// * `record`: 包含字段名和值的哈希映射
/// * `ignore_fields`: 需要忽略的字段列表
/// * `line`: 是否按行格式化(True则每个字段换行，False则空格分隔)
///
/// # 返回值
/// 返回格式化后的字符串
fn format_content(record: &HashMap<String, Value>, ignore_fields: &Vec<String>, line: bool) -> String {
    let mut content = String::new();
    // 遍历记录中的每个键值对
    for (key, value) in record {
        // 跳过需要忽略的字段
        if ignore_fields.contains(&key.to_string()) { continue; }
        match value {
            // 处理字符串类型的值
            Value::String(str_value) => {
                content.push_str(&format!("{}: {}{}", key, str_value, if line { "\n" } else { " " }));
            }
            // 处理其他类型的值(数字、布尔值等)
            _ => {
                content.push_str(&format!("{}: {}{}", key, value.to_string(), if line { "\n" } else { " " }));
            }
        }
    }
    content
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use common::log::init_tk_log;
    use common::yaml::{DsConfig};
    use database::core::DBConn;
    use crate::core::vector::init_vector_store;
    use crate::core::vector_data_etl::init_collection_documents;

    #[test]
    fn test_add_collection_documents() {
        init_tk_log();

        tokio_test::block_on(async move {
            let collection_name = "ecommerce-1";
            let source_type = "order";
            let data_config:Value = serde_json::json!({
              "database": "ecommerce",
              "table": "order",
              "column": "id, customer_id, order_date, status, total_amount, shipping_address, @item_list",
              "@item_list": {
                "title": "商品列表",
                "sql": "SELECT oi.order_id as id, i.name as item_name, oi.quantity as quantity, oi.price_per_item as price_per_item FROM ecommerce.order_items oi JOIN ecommerce.item i ON oi.item_id = i.id WHERE oi.order_id IN (?id)"
              },
              "metadata": {
                "id": "order_id"
              }
            });
            
            init_vector_store(collection_name).await;

            if let Ok(mysql_url) = std::env::var("MYSQL_URL") {
                let db_conn = DBConn::new(DsConfig { url: mysql_url }).await.unwrap();
                init_collection_documents(&db_conn, collection_name, source_type, data_config).await;
            } else {
                eprintln!("Error getting MySQL_URL");
            }
        });
    }
}
