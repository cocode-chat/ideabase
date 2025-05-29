use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use langchain_rust::vectorstore::{qdrant::{Qdrant, Store, QdrantClient, StoreBuilder}, VectorStore};

use crate::core::llm::get_embedder_llm;
use crate::core::chain::{build_retriever_chain, RETRIEVER_CHAINS};

// 全局缓存，使用 Arc<VectorStore> 存储向量存储实例
pub static VECTOR_STORES: Lazy<Mutex<HashMap<String, Arc<dyn VectorStore>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

/// 初始化向量存储并添加到缓存中
/// 
/// # 参数
/// 
/// * `collection_name` - 集合名称
/// * `vector_store` - 向量存储实例
pub async fn init_vector_store(collection_name: &str) {
    if let Ok(store) = build_vector_store(collection_name).await {
        let mut stores = VECTOR_STORES.lock().unwrap();
        stores.insert(collection_name.to_string(), Arc::new(store));
    } else {
        log::error!("Couldn't create vector store for Collection {}", collection_name);
    }
    if let Ok(chain) = build_retriever_chain(collection_name).await {
        let mut chains = RETRIEVER_CHAINS.lock().unwrap();
        chains.insert(collection_name.to_string(), chain);
    } else {
        log::error!("Couldn't create retriever chain for Collection {}", collection_name);
    }
}

/// 从缓存中获取向量存储
/// 
/// # 参数
/// 
/// * `collection_name` - 集合名称
/// 
/// # 返回值
/// 
/// 如果缓存中存在该集合名称对应的向量存储，则返回 Some(Arc<VectorStore>)，否则返回 None
pub fn get_vector_store(collection_name: &str) -> Option<Arc<dyn VectorStore>> {
    let stores = VECTOR_STORES.lock().unwrap();
    stores.get(collection_name).cloned()
}


/// 构建并返回一个Qdrant向量数据库客户端
///
/// # 参数
/// * `collection_name` - 要操作的集合名称
/// * `vector` - 向量数据库配置，包含连接信息
/// * `embedding` - 嵌入模型配置，用于文本向量化
///
/// # 返回值
/// 返回一个配置好的Qdrant向量存储客户端(Store)
pub async fn build_vector_store(collection_name: &str) -> Result<Store, String> {
    // 创建Qdrant客户端实例
    let vector_db_client = get_vector_client()?;
    // 初始化OpenAI嵌入器(Embedder)
    let embedder = get_embedder_llm()?;
    // 构建向量存储客户端
    let vector_db_store = StoreBuilder::new()
        .embedder(embedder)           // 设置嵌入器
        .client(vector_db_client)          // 设置Qdrant客户端
        .collection_name(collection_name)  // 设置集合名称
        .build()
        .await;
    match vector_db_store {
        Ok(vector_db_store) => { Ok(vector_db_store) }
        Err(err) => {
            log::error!("Couldn't build vector store: {} {}", collection_name, err);
            Err(format!("build_vector_store.err: {}", err))
        }
    }
}

/// 获取Qdrant向量数据库客户端
/// 
/// 从环境变量中读取VECTOR_DB_URL和VECTOR_API_KEY配置，
/// 创建并返回Qdrant客户端实例
/// 
/// # 返回值
/// 返回Result类型，成功时包含Qdrant客户端实例，失败时返回错误信息
pub fn get_vector_client() -> Result<Qdrant, String> {
    // 从环境变量获取向量数据库URL
    let vector_db_url = std::env::var("VECTOR_DB_URL")
        .map_err(|_| "Missing VECTOR_DB_URL environment variable")?;
    
    // 从环境变量获取API密钥    
    let api_key = std::env::var("VECTOR_API_KEY")
        .map_err(|_| "Missing VECTOR_API_KEY environment variable")?;

    // 创建Qdrant客户端实例
    let vector_db_client = QdrantClient::from_url(&vector_db_url)
        .api_key(api_key)
        .build()
        .unwrap();
        
    // 返回客户端实例
    Ok(vector_db_client)
}