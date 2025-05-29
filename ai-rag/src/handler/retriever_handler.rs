use langchain_rust::schemas::Document;
use langchain_rust::vectorstore::VecStoreOptions;
use crate::core::vector::VECTOR_STORES;

/// 执行向量相似性搜索
/// 
/// # 参数
/// * `query`: 搜索查询字符串
/// * `collection_name`: 向量集合名称
/// 
/// # 返回值
/// 返回Result类型，成功时包含匹配的文档向量，失败时返回错误信息
pub async fn similarity_search(query: &str, collection_name: &str) -> Result<Vec<Document>, String> {
    // 获取指定集合的向量存储
    let vector_store = VECTOR_STORES.lock().unwrap().get(collection_name).unwrap().clone();
    // 执行相似性搜索，返回最相似的3个文档
    let query_result = vector_store.similarity_search(&query, 3, &VecStoreOptions::default()).await;
    // 处理搜索结果
    match query_result {
        Ok(docs) => { Ok(docs) },
        Err(err) => {
            // 记录错误日志并返回错误信息
            log::error!("Error searching documents: {} {} {:?}",collection_name, query, err);
            Err(format!("Error vector search: {:?}", err))
        }
    }
}
