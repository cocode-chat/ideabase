use database::core::DBConn;
use common::yaml::load_env_json;
use crate::core::vector::{get_vector_client, init_vector_store};
use crate::core::vector_doc::init_collection_documents;

/// 初始化向量数据库
/// 
/// 从vector.json配置文件中读取配置，初始化所有指定的向量集合
/// 对于每个集合，会先删除旧集合(如果存在)，然后创建新集合并初始化文档数据
/// 
/// # 参数
/// * `db_conn`: 数据库连接对象
pub async fn init_vector_db(db_conn: &DBConn) {
    // 从环境变量加载vector.json配置文件
    let vector_value = load_env_json("vector.json");
    
    // 检查配置是否为有效的JSON对象
    if let Some(vector_map) = vector_value.as_object() {
        // 遍历所有向量集合配置
        for (collection_name, vector_data) in vector_map {
            log::info!("init vector db, collection_name: {}", collection_name);
            
            // 1. 删除已存在的集合
            delete_collection(collection_name).await;
            
            // 2. 初始化新的向量存储
            init_vector_store(collection_name).await;
            
            // 3. 初始化集合中的文档数据
            for (source_type, vector_config) in vector_data.as_object().unwrap() {
                init_collection_documents(&db_conn, collection_name, source_type, vector_config.clone()).await;
            }
        }
    } else {
        // 配置文件解析失败处理
        log::error!("vector.json parse error, {}", vector_value);
    }
}

/// 删除向量数据库中的集合
/// 
/// # 参数
/// * `collection_name`: 要删除的集合名称
async fn delete_collection(collection_name: &str) {
    // 获取向量数据库客户端
    if let Ok(client) = get_vector_client() {
        // 执行删除操作
        client.delete_collection(collection_name).await.expect("delete collection error");
        log::info!("delete collection success: {}", collection_name);
    } else {
        log::error!("Couldn't get vector client for delete_collection: {}", collection_name);
    }
}