use actix_web::{post, web, Responder};
use http::StatusCode;

use common::passwd::hash_passwd;
use common::rpc::RpcResult;
use crate::G_DB;
use crate::controllers::build_rpc_response;
use crate::service::model::account::{Account, AccountDTO};

#[post("/auth/account.json")]
pub async fn create_account(request_data: web::Json<AccountDTO>) -> impl Responder {
    let mut account_dto = request_data.into_inner();

    // 使用模式匹配进行参数验证
    match (account_dto.email.as_ref(), account_dto.phone.as_ref(), account_dto.password.as_ref()) {
        (None, None, _) => return build_rpc_response(
            RpcResult {
                code: StatusCode::BAD_REQUEST,
                msg: Some("email or phone must be provided".to_string()),
                payload: Some(-1)
            }
        ),
        (_, _, None) => return build_rpc_response(
            RpcResult {
                code: StatusCode::BAD_REQUEST,
                msg: Some("password must be provided".to_string()),
                payload: Some(-1)
            }
        ),
        _ => ()
    }

    // 加密密码并处理可能的错误
    let hashed_passwd = match hash_passwd(&account_dto.password.unwrap()) {
        Ok(hashed) => hashed,
        Err(err) => {
            return build_rpc_response(RpcResult {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                msg: Some(format!("Password hashing failed: {}", err)),
                payload: Some(-1)
            });
        }
    };
    account_dto.password = Some(hashed_passwd);

    // 写数据并返回
    let db_conn = G_DB.get().unwrap();
    match Account::create(db_conn, &account_dto).await {
        Ok(account_id) => build_rpc_response(RpcResult {
            code: StatusCode::OK,
            msg: None,
            payload: Some(account_id)
        }),
        Err(err) => build_rpc_response(RpcResult {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: Some(err.to_string()),
            payload: Some(-1)
        })
    }
}