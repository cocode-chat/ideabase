use serde::{Deserialize, Serialize};
use actix_web::{post, web, Responder};
use http::StatusCode;
use common::rpc::RpcResult;
use rag::handler::conversation_handler::conversation_call;
use rag::handler::retriever_handler::similarity_search;
use crate::controllers::build_rpc_response;

pub fn scope() -> actix_web::Scope {
    web::scope("/ai").service(recall).service(conversation)
}

#[post("/conversation.json")]
async fn conversation(request_data: web::Json<Conversation>) -> impl Responder {
    let request_data = request_data.into_inner();
    let collection_name = request_data.collection;
    let message = request_data.message;
    let answer = conversation_call(&message, &collection_name).await;
    match answer {
        Ok(answer) => {
            build_rpc_response(RpcResult { code: StatusCode::OK, msg: None, payload: Some(answer) })
        }
        Err(err) => {
            build_rpc_response(RpcResult { code: StatusCode::INTERNAL_SERVER_ERROR, msg: Some(err), payload: None})
        }
    }
}

#[post("/rag/recall.json")]
async fn recall(request_data: web::Json<Conversation>) -> impl Responder {
    let request_data = request_data.into_inner();
    let collection_name = request_data.collection;
    let message = request_data.message;
    let result = similarity_search(&message, &collection_name).await;
    match result {
        Ok(docs) => {
            build_rpc_response(RpcResult { code: StatusCode::OK, msg: None, payload: Some(docs) })
        }
        Err(err) => {
            build_rpc_response(RpcResult { code: StatusCode::INTERNAL_SERVER_ERROR, msg: Some(err), payload: None})
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
struct Conversation {
    collection: String,
    message: String,
}
