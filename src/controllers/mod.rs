pub mod rag_controller;
pub mod restful_controller;
pub mod account_controller;

use actix_web::{get, http::StatusCode, web, HttpResponse, Responder};
use common::rpc::RpcResult;
use crate::controllers::rag_controller::{conversation, recall};
use crate::controllers::restful_controller::{curd, get_table_meta, get_table_names};

// 快速返回结果
pub fn return_rpc_result<T: serde::Serialize>(code: u16, msg: Option<String>, data: Option<T>) -> impl Responder {
    HttpResponse::Ok().json(RpcResult{code, msg, data})
}

pub fn configure_cors() -> actix_cors::Cors {
    actix_cors::Cors::default()
        .allowed_origin("https://ideabase.io")
        .allowed_methods(vec!["*"])
        .allowed_headers(vec!["content-type"])
        .supports_credentials()
        .max_age(3600)
}

#[get("/health")]
async fn health() -> impl Responder {
    return_rpc_result(StatusCode::OK.as_u16(), None, Some("I'm OK!"))
}

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    //health
    cfg.service(health);

    // api scope
    let api_scope = web::scope("/api/v1")
        .service(curd)
        .service(get_table_names).service(get_table_meta)
        .service(recall).service(conversation);
    cfg.service(api_scope);
}

