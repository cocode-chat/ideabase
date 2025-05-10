use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionEntity {
    // 数据表
    pub table: String,
    // 数据表列 英文,号分割
    pub column: String,
    // 写入向量库的元数据
    pub metadata: HashMap<String, String>,
}
