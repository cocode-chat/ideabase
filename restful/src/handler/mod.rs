pub mod head;
pub mod get;
pub mod put;
pub mod post;
pub mod delete;

/// 构建标准化的HTTP处理结果
///
/// # 参数
/// * `success` - 操作是否成功的标志
/// * `result_map` - 包含各表处理结果的HashMap，key为表名，value为处理结果
///
/// # 返回值
/// 返回标准化的JSON响应，格式为：
/// * 成功时：`{"code": 200, "data": {各表结果}}`
/// * 失败时：`{"code": 400, "msg": "failure", "data": {各表结果}}``
pub fn build_rpc_value(code: u32, msg: Option<String>, result_map: std::collections::HashMap<String, serde_json::Value>) -> serde_json::Value {
    let result_data = serde_json::Value::Object(result_map.into_iter().collect());
    match msg {
        Some(msg) => {
            serde_json::json!({ "code": code, "msg": msg, "data": result_data })
        }
        None => { serde_json::json!({ "code": code, "data": result_data }) }
    }
}