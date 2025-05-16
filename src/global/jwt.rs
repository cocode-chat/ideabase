use std::pin::Pin;
use std::future::Future;
use std::time::{ SystemTime, UNIX_EPOCH };
use serde::{Deserialize, Serialize};
use actix_web::{ dev::Payload, FromRequest, HttpRequest, error::ErrorUnauthorized, Error as ActixError};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation, errors::Error };
use crate::G_ENV;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtToken {
    sub: i64,      // Subject (e.g., user ID)
    role: String,  // Role (e.g., "admin", "user")
    exp: u128,     // Expiration time (as UTC timestamp)
}

impl FromRequest for JwtToken {
    type Error = ActixError;
    type Future = Pin<Box<dyn Future<Output = Result<JwtToken, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let path = req.uri().to_string();
        // 先将header内容提取出来，避免生命周期问题
        let auth_header = req.headers().get("Authentication").and_then(|h| h.to_str().ok()).map(|s| s.to_owned());

        Box::pin(async move {
            let token = match auth_header
                .as_deref()
                .and_then(|h| {
                    let mut parts = h.split_whitespace();
                    match (parts.next(), parts.next()) {
                        (Some("Bearer"), Some(token)) => Some(token),
                        _ => None,
                    }
                }) {
                Some(token) => token,
                None => {
                    log::error!("Authentication header miss or format error, path: {}", path);
                    return Err(ErrorUnauthorized("Unauthorized"));
                }
            };
            // 验证token
            let secret = &G_ENV.jwt.secret;
            match JwtToken::verify(token, secret) {
                Ok(jwt_token) => Ok(jwt_token),
                Err(err) => {
                    log::error!("Authorization token verify error, path: {} token: {} {:?}", path, token, err);
                    Err(ErrorUnauthorized("Unauthorized"))
                }
            }
        })
    }
}


impl JwtToken {

    pub fn new(sub: i64, role: &str, exp_hour: u32) -> JwtToken {
        let exp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() + (exp_hour * 3600 * 1000) as u128;
        JwtToken { sub, role: role.to_string(), exp }
    }

    /// create jwt token
    pub fn create_token(&self, secret: &str) -> Result<String, Error> {
        encode(&Header::default(), self, &EncodingKey::from_secret(secret.as_ref()))
            .map_err(Into::into)
    }

    /// verify jwt token
    pub fn verify(token: &str, secret: &str) -> Result<JwtToken, Error> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_required_spec_claims(&["sub", "role", "exp"]);
        decode::<Self>(token, &DecodingKey::from_secret(secret.as_ref()), &validation)
            .map(|c| c.claims)
            .map_err(Into::into)
    }
}



#[cfg(test)]
mod tests {
    use common::log::init_tk_log;
    use crate::global::jwt::JwtToken;

    #[test]
    fn test_jwt() {
        init_tk_log();

        let secret = "secret";
        let sub = 1270790813134464;
        let role = "admin";
        let jwt = JwtToken::new(sub, role, 3u32);
        let res = jwt.create_token(secret).unwrap();
        log::info!("token.0: {:?}",res);
        let token = JwtToken::verify(&res, secret);
        log::info!("token.1: {:?}",token)
    }
}