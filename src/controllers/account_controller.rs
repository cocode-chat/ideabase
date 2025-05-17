use actix_web::{post, web, Responder};
use http::StatusCode;
use regex::Regex; 
use lazy_static::lazy_static;

use common::passwd::hash_passwd;
use common::rpc::RpcResult;
use serde::{Deserialize, Serialize};
use common::utils::generate_api_key;
use crate::G_DB;
use crate::controllers::build_rpc_response;
use crate::service::model::account::{Account, AccountDTO, Role};

lazy_static! {
    // 定义邮箱验证的正则表达式
    static ref EMAIL_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    // 定义密码验证的正则表达式：至少包含一个大写字母、一个小写字母和一个数字
    static ref PASSWORD_REGEX: Regex = Regex::new(r"^(?=.*[A-Z])(?=.*[a-z])(?=.*\d).+$").unwrap();
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AccountRequest {
    pub id: Option<i64>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub role: Option<String>,
}

// 创建普通用户
#[post("/auth/account.json")]
pub async fn create_account(request_data: web::Json<AccountRequest>) -> impl Responder {
    let mut account_request = request_data.into_inner();

    // 使用模式匹配进行参数验证
    match (account_request.email.as_ref(), account_request.password.as_ref()) {
        (None, _) => return build_rpc_response(
            RpcResult { code: StatusCode::BAD_REQUEST, msg: Some("email must be provided".to_string()), payload: None }
        ),
        (_, None) => return build_rpc_response(
            RpcResult { code: StatusCode::BAD_REQUEST, msg: Some("password must be provided".to_string()), payload: None }
        ),
        (Some(email), Some(password)) => {
            // 邮箱格式验证
            if !EMAIL_REGEX.is_match(email) {
                return build_rpc_response(
                    RpcResult { code: StatusCode::BAD_REQUEST, msg: Some("invalid email format".to_string()), payload: None }
                );
            }
            // 验证密码复杂度
            if !PASSWORD_REGEX.is_match(password) {
                return build_rpc_response(
                    RpcResult { code: StatusCode::BAD_REQUEST, msg: Some("password must contain uppercase letters, lowercase letters, and numbers".to_string()), payload: None }
                );
            }
        }
    }

    // 加密密码并处理可能的错误
    let hashed_passwd = match hash_passwd(&account_request.password.unwrap()) {
        Ok(hashed) => hashed,
        Err(err) => {
            return build_rpc_response(RpcResult {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                msg: Some(format!("Password hashing failed: {}", err)),
                payload: Some(-1)
            });
        }
    };
    account_request.password = Some(hashed_passwd);
    account_request.role = Some(Role::User.to_string());

    // 写数据并返回
    let db_conn = G_DB.get().unwrap();
    // 把 account_request 转为 AccountDTO
    let account_dto = AccountDTO {
        email: account_request.email.clone(),
        password: account_request.password.clone(),
        role: account_request.role.clone(),
        api_key: None, email_confirmed_at: None, last_sign_in_at: None, gmt_create: None, gmt_update: None,
    };
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

// 为用户生成 api_key
#[post("/auth/account/api_key.json")]
pub async fn gen_account_api_key(request_data: web::Json<AccountRequest>) -> impl Responder {
    let account_request = request_data.into_inner();
    
    // 验证用户ID是否存在
    if account_request.id.is_none() {
        return build_rpc_response(
            RpcResult { code: StatusCode::BAD_REQUEST, msg: Some("account id must be provided".to_string()), payload: None }
        );
    }
    
    let account_id = account_request.id.unwrap();
    let db_conn = G_DB.get().unwrap();
    
    // 检查用户是否存在
    match Account::fetch_by_id(db_conn, account_id).await {
        Ok(_) => {
            let api_key = generate_api_key(account_id);
            let update_dto = AccountDTO { api_key: Some((&api_key).to_string()) ,
                email: None, password: None, role: None, email_confirmed_at: None, last_sign_in_at: None, gmt_create: None, gmt_update: None
            };
            match Account::update(db_conn, account_id, &update_dto).await {
                Ok(_) => build_rpc_response(RpcResult { code: StatusCode::OK, msg: None, payload: Some(api_key) }),
                Err(err) => build_rpc_response(RpcResult { code: StatusCode::INTERNAL_SERVER_ERROR, msg: Some(format!("update api_key error, {} {}", account_id, err)), payload: None })
            }
        },
        Err(err) => build_rpc_response(RpcResult { code: StatusCode::INTERNAL_SERVER_ERROR, msg: Some(format!("account fetch error: {}", err)), payload: None })
    }
}
