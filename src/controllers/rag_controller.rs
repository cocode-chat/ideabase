use serde::{Deserialize, Serialize};
use actix_web::{post, web, HttpResponse, Responder};
use common::rpc::RpcResult;
use rag::handler::conversation_handler::conversation_call;
use rag::handler::retriever_handler::similarity_search;

#[derive(Serialize, Deserialize, Debug)]
struct Conversation {
    collection: String,
    message: String,
}

#[post("/rag/conversation.json")]
pub async fn conversation(request_data: web::Json<Conversation>) -> impl Responder {
    let request_data = request_data.into_inner();
    let collection_name = request_data.collection;
    let message = request_data.message;
    let answer = conversation_call(&message, &collection_name).await;
    match answer {
        Ok(answer) => {
            HttpResponse::Ok().json(RpcResult { code: 200, msg: None, data: Some(answer) })
        }
        Err(err) => {
            HttpResponse::Ok().json(RpcResult { code: 500, msg: Some(err.to_string()), data: Some(serde_json::Value::Null) })
        }
    }
}

#[post("/rag/recall.json")]
pub async fn recall(request_data: web::Json<Conversation>) -> impl Responder {
    let request_data = request_data.into_inner();
    let collection_name = request_data.collection;
    let message = request_data.message;
    let result = similarity_search(&message, &collection_name).await;
    match result {
        Ok(docs) => {
            for doc in &docs {
                println!("doc: {}", serde_json::to_string_pretty(&doc).unwrap());
            }
            HttpResponse::Ok().json(RpcResult { code: 200, msg: None, data: Some(docs) })
        }
        Err(err) => {
            HttpResponse::Ok().json(RpcResult { code: 500, msg: Some(err.to_string()), data: Some(serde_json::Value::Null) })
        }
    }
}