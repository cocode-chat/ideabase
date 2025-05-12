use std::collections::HashMap;
use actix_web::{get, post, web, HttpResponse, Responder};

use crate::G_DB;
use common::rpc::RpcResult;
use database::core::{get_table, get_table_name_list};
use restful::handler::delete::handle_delete;
use restful::handler::get::handle_get;
use restful::handler::head::handle_head;
use restful::handler::post::handle_post;
use restful::handler::put::handle_put;

/// 处理CRUD操作的REST API端点
///
/// # 参数
/// * `proj_id` - 项目ID
/// * `action` - 操作类型(get/post/put/delete)
/// * `request_data` - JSON格式的请求数据
///
/// # 返回值
/// 返回JSON格式的响应数据，包含操作结果或错误信息
#[post("/rest/{method}.json")]
pub async fn curd(params: web::Path<String>, request_data: web::Json<HashMap<String, serde_json::Value>>) -> impl Responder {
    let method = params.into_inner();
    let request_data = request_data.into_inner();
    log::debug!("crud: {} {}", &method, serde_json::to_string_pretty(&request_data).unwrap());
    let result_value: serde_json::Value;
    match method.as_str() {
        "head" => {
            let db = G_DB.get().unwrap();
            result_value = handle_head(db, request_data).await;
        },
        "get" => {
            let db = G_DB.get().unwrap();
            result_value = handle_get(db, request_data).await;
        }
        "put" => {
            let db = G_DB.get().unwrap();
            result_value = handle_put(db, request_data).await;
        }
        "post" => {
            let db = G_DB.get().unwrap();
            result_value = handle_post(db, request_data).await;
        }
        "delete" => {
            let db = G_DB.get().unwrap();
            result_value = handle_delete(db, request_data).await;
        }
        _ => {
            result_value = serde_json::json!({ "code": 400, "msg": format!("unknown method: {}", method) });
        }
    }
    HttpResponse::Ok().json(result_value)
}

#[get("/rest/{schema}/tables.json")]
pub async fn get_table_names(schema: web::Path<String>) -> impl Responder {
    let schema = schema.into_inner();
    let table_name_map = get_table_name_list(&schema);
    HttpResponse::Ok().json(RpcResult { code: 200, msg: None, data: Some(table_name_map) })
}

#[get("/rest/{schema}/{table}.json")]
pub async fn get_table_meta(params: web::Path<(String, String)>) -> impl Responder {
    let (schema, table) = params.into_inner();
    let table_mata_opt = get_table(&schema, &table);
    let mut rpc_result = RpcResult { code: 200, msg: None, data: None };
    match table_mata_opt {
        Some(table_mata) => {
            rpc_result.data = Some(table_mata);
        }
        None => {
            rpc_result.code = 400;
            rpc_result.msg = Some("table not found".to_string());
        }
    }
    HttpResponse::Ok().json(rpc_result)
}