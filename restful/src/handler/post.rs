use std::collections::HashMap;

use common::utils::get_next_id;
use database::core::{is_table_exists, DBConn};
use crate::handler::build_rpc_value;

/// 处理数据插入请求
/// 
/// # 参数
/// * `body_map` - 包含插入请求的数据映射，key为表名，value为要插入的数据
/// 
/// # 返回值
/// 返回 JSON 格式的处理结果：
/// * 成功：返回插入后的完整记录数据
/// * 失败：`{"code": 400, "msg": "错误信息"}`
/// 
/// # 示例
/// ```json
/// {
///   "user": {
///     "name": "张三",
///     "age": 25,
///     "email": "zhangsan@example.com"
///   }
/// }
/// ```
pub async fn handle_post(db: &DBConn, body_map: HashMap<String, serde_json::Value>) -> serde_json::Value {
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

                // 写入数据
                match insert_one(db, &schema, &table, param_map).await {
                    Ok(id) => {
                        result_map.insert(table_key.clone(), serde_json::json!(id));
                    },
                    Err(err) => {
                        code = 400;
                        result_map.insert(table_key.clone(), serde_json::Value::String(err));
                    }
                }
            }
            None => {
                code = 400;
                result_map.insert(table_key.clone(), serde_json::Value::String(format!("参数格式错误，value: {:?}", param)));
            }
        }
    }
    build_rpc_value(code, None, result_map)
}

/// 执行单条记录的插入操作
/// 
/// # 参数
/// * `db` - 数据库连接实例
/// * `table` - 要插入数据的表名
/// * `kvs` - 包含要插入的字段和值的键值对映射
/// 
/// # 返回值
/// * `Ok(i64)` - 成功时返回插入记录的 ID
/// * `Err(String)` - 失败时返回错误信息
/// 
/// # 实现细节
/// 将传入的键值对转换为 SQL INSERT 语句，格式为：
/// ```sql
/// INSERT INTO table_name(field1,field2) VALUES(value1,value2)
/// ```
async fn insert_one(db: &DBConn, schema: &str, table: &str, kvs: &serde_json::Map<String, serde_json::Value>) -> Result<i64, String> {
    let data_id = get_next_id();
    let mut fields = Vec::new();
    let mut values = Vec::new();

    // 自动生成 ID
    fields.push("id");
    values.push(data_id.to_string());
    for (field, value) in kvs {
        fields.push(field.as_str());
        values.push(value.to_string());
    }
    let sql = format!("INSERT INTO {}.{}({}) VALUES({})", schema, table, fields.join(","), values.join(","));
    match db.insert(&sql).await {
        Ok(cnt) => {
            let result_id = if cnt > 0 { data_id as i64 } else { -1i64 };
            Ok(result_id)
        },
        Err(e) => Err(e.to_string())
    }
}