use database::core::DBConn;
use common::yaml::load_env_json;
use crate::core::vector::{get_vector_client, init_vector_store};
use crate::core::vector_data_etl::init_collection_documents;

pub async fn init_vector_db(db_conn: &DBConn) {
    let vector_value = load_env_json("vector.json");
    if let Some(vector_map) = vector_value.as_object() {
        for (collection_name, vector_data) in vector_map {
            log::info!("init vector db, collection_name: {}", collection_name);
            // 删除集合
            delete_collection(collection_name).await;
            // 初始化 vector store
            init_vector_store(collection_name).await;
            // 初始化向量数据
            for (source_type, vector_config) in vector_data.as_object().unwrap() {
                init_collection_documents(&db_conn, collection_name, source_type, vector_config.clone()).await;
            }
        }
    } else {
        log::error!("vector.json parse error, {}", vector_value);
    }
}

// 删除 vector db collection
async fn delete_collection(collection_name: &str) {
    if let Ok(client) = get_vector_client() {
        client.delete_collection(collection_name).await.expect("delete collection error");
        log::info!("delete collection success: {}", collection_name);
    } else {
        log::error!("Couldn't get vector client for delete_collection: {}", collection_name);
    }
}