use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct RpcResult<T: Serialize> {
    pub code: u16, // 状态码 -> http status code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>, // 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>, // 数据
}