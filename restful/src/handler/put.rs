use std::collections::HashMap;
use database::core::{is_table_exists, DBConn};
use crate::handler::build_rpc_value;

/// 处理数据更新请求
/// 
/// # 参数
/// * `body_map` - 包含更新请求的数据映射，key为表名，value为更新数据
/// 
/// # 返回值
/// 返回 JSON 格式的处理结果：
/// * 成功：返回更新后的完整记录数据
/// * 失败：`{"code": 400, "msg": "错误信息"}`
/// 
/// # 示例
/// ```json
/// {
///   "user": {
///     "id": 1,              // 必须包含 id 字段
///     "name": "新名字",      // 要更新的字段
///     "age": 25
///   }
/// }
/// ```
pub async fn handle_put(db: &DBConn, body_map: HashMap<String, serde_json::Value>) -> serde_json::Value {
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
                    let err_msg = format!("{}'s database should be specified", &table_key);
                    result_map.insert((&table_key).to_string(), serde_json::json!(err_msg));
                    continue;
                }
                // 检查表是否存在，不存在则记录错误
                if !is_table_exists(&schema, &table) {
                    code = 400;
                    let err_msg = format!("table {} not exists", &table_key);
                    result_map.insert((&table_key).to_string(), serde_json::json!(err_msg));
                    continue;
                }

                // 更新数据
                match update_one(db, &schema, &table, param_map).await {
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

// updateOne 执行单条记录的更新操作
// 参数：
//   - table: 要更新的表名
//   - kvs: 包含更新字段和值的键值对映射，必须包含 id 字段
//
// 返回：
//   - int64: 更新记录的 id，如果出错则返回负数错误码
//   - error: 错误信息，如果成功则为 nil
pub async fn update_one(db: &DBConn, schema: &str, table: &str, kvs: &serde_json::Map<String, serde_json::Value>) -> Result<i64, String> {
    if let Some(id_value) = kvs.get("id") {
        // 检查 id 是否为数字类型
        if !id_value.is_number() {
            return Err(format!("'id' type is not num, key: {}, kvs: {:?}", table, kvs));
        }
        let id = id_value.as_i64().unwrap();
        // 构建更新字段和参数
        let mut fields = Vec::new();
        for (k, v) in kvs.iter() {
            if k != "id" {
                fields.push(format!("`{}`={}", k, v));
            }
        }
        let sql = format!("update {}.{} set {} where id={}", schema, table, fields.join(","), id);
        match db.update(&sql).await {
            Ok(cnt) => if cnt > 0 { Ok(id) } else { Ok(-1) },
            Err(e) => Err(e.to_string())
        }
    } else {
        Err(format!("data update must have 'id' field, key: {}, kvs: {:?}", table, kvs))
    }
}