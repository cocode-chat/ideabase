use langchain_rust::{ prompt_args };
use crate::core::llm::RETRIEVER_CHAINS;

pub async fn conversation_call(message: &str, collection_name: &str) -> Result<String, String> {
    let input_variables = prompt_args! {
        "question" => message,
    };
    let chain = RETRIEVER_CHAINS.lock().unwrap().get(collection_name).unwrap().clone();
    let result = chain.invoke(input_variables).await;
    match result {
        Ok(message) => {
            Ok(message)
        }
        Err(err) => {
            log::error!("Error invoking chain: {} {:?}", collection_name, err);
            Err(format!("Error vector search: {:?}", err))
        }
    }
}