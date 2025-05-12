use langchain_rust::schemas::Document;
use langchain_rust::vectorstore::VecStoreOptions;
use crate::core::vector::VECTOR_STORES;

pub async fn similarity_search(query: &str, collection_name: &str) -> Result<Vec<Document>, String> {
    let vector_store = VECTOR_STORES.lock().unwrap().get(collection_name).unwrap().clone();
    let query_result = vector_store.similarity_search(&query, 3, &VecStoreOptions::default()).await;
    match query_result {
        Ok(docs) => { Ok(docs) },
        Err(err) => {
            log::error!("Error searching documents: {} {} {:?}",collection_name, query, err);
            Err(format!("Error vector search: {:?}", err))
        }
    }
}
