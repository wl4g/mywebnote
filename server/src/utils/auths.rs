use lazy_static::lazy_static;
use std::{ collections::HashMap, sync::Arc };

use axum::body::Body;
use chrono::{ Duration, Utc };
use hyper::{ HeaderMap, Response, StatusCode };
use jsonwebtoken::{ decode, encode, DecodingKey, EncodingKey, Header, Validation };
use serde::{ Deserialize, Serialize };
use tower_cookies::cookie::Cookie;
use tokio::sync::RwLock;

use crate::{
    config::config_api::ApiConfig,
    types::auths::{ LoggedResponse, TokenWrapper },
    utils::webs,
};

lazy_static! {
    // singleton instance.
    static ref SECURITY_CONTEXT: Arc<SecurityContext> = Arc::new(SecurityContext::new());
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthUserClaims {
    pub sub: String,
    pub uname: String,
    pub email: String,
    pub exp: usize,
    pub ext: Option<HashMap<String, String>>,
}

pub fn create_jwt(
    config: &Arc<ApiConfig>,
    uid: &str,
    uname: &str,
    email: &str,
    is_refresh: bool,
    extra_claims: Option<HashMap<String, String>>
) -> String {
    let expiration = Utc::now()
        .checked_add_signed(
            Duration::milliseconds(
                if is_refresh {
                    config.auth.jwt_validity_rk.unwrap() as i64
                } else {
                    config.auth.jwt_validity_ak.unwrap() as i64
                }
            )
        )
        .expect("valid timestamp")
        .timestamp();

    let claims = AuthUserClaims {
        sub: uid.to_owned(),
        uname: uname.to_owned(),
        email: email.to_owned(),
        exp: expiration as usize,
        ext: extra_claims,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.auth.jwt_secret.to_owned().unwrap().as_ref())
    ).expect("failed to encode jwt")
}

pub fn validate_jwt(
    config: &Arc<ApiConfig>,
    token: &str
) -> Result<AuthUserClaims, jsonwebtoken::errors::Error> {
    let validation = Validation::default();
    let token_data = decode::<AuthUserClaims>(
        token,
        &DecodingKey::from_secret(config.auth.jwt_secret.to_owned().unwrap().as_ref()),
        &validation
    )?;
    Ok(token_data.claims)
}

pub fn auth_resp_redirect_or_json(
    config: &Arc<ApiConfig>,
    headers: &HeaderMap,
    redirect_url: &str,
    status: StatusCode,
    message: &str,
    cookies: Option<(Option<Cookie>, Option<Cookie>, Option<Cookie>)>
) -> Response<Body> {
    let (ak, rk, _) = match &cookies {
        Some(triple) => {
            (
                triple.to_owned().0.map(|c| TokenWrapper {
                    value: c.value().to_string(),
                    expires_in: config.auth.jwt_validity_ak.unwrap(),
                }),
                triple.to_owned().1.map(|c| TokenWrapper {
                    value: c.value().to_string(),
                    expires_in: config.auth.jwt_validity_rk.unwrap(),
                }),
                triple.2.to_owned(),
            )
        }
        None => (None, None, None),
    };

    let json = LoggedResponse {
        errcode: status.as_u16() as i16,
        errmsg: message.to_string(),
        access_token: ak,
        refresh_token: rk,
        redirect_url: Some(redirect_url.to_owned()),
    };
    let json_str = serde_json::to_string(&json).unwrap();

    webs::response_redirect_or_json(status, headers, cookies, redirect_url, &message, &json_str)
}

#[derive(Clone, Debug)]
pub struct SecurityContext {
    pub current_user: Arc<RwLock<Option<AuthUserClaims>>>,
}

impl SecurityContext {
    pub fn new() -> Self {
        SecurityContext {
            current_user: Arc::new(RwLock::new(None)),
        }
    }

    pub fn get_instance() -> Arc<SecurityContext> {
        SECURITY_CONTEXT.clone()
    }

    pub async fn bind(&self, user: Option<AuthUserClaims>) {
        tracing::debug!("Binding from user: {:?}", user);
        match user {
            Some(user) => {
                // Notice: 必须在此函数中执行 write() 获取写锁, 若在外部 routes/auths.rs#auth_middleware() 中获取写锁,
                // 则当在 routes/users.rs#handle_get_users() 中获取读锁时会产生死锁, 因为 RwLock 的释放机制是超出作用域自动释放,
                // 在 auth_middleware() 中写锁的生命周期包含了 handle_get_users() 即没有释放.
                let mut current_user = self.current_user.write().await;
                *current_user = Some(user);
            }
            None => {}
        }
        tracing::debug!("Binded from user: {:?}", self.get().await);
    }

    pub async fn get(&self) -> Option<AuthUserClaims> {
        match self.current_user.try_read() {
            Ok(read_guard) => read_guard.clone(),
            Err(e) => {
                tracing::error!("Unable to acquire read lock. reason: {:?}", e);
                None
            }
        }
    }

    pub async fn get_current_uid(&self) -> Option<String> {
        match self.get().await {
            Some(claims) => Some(claims.sub),
            None => {
                tracing::error!("No found current user claims sub.");
                None
            }
        }
    }

    pub async fn get_current_uname(&self) -> Option<String> {
        match self.get().await {
            Some(claims) => Some(claims.uname),
            None => {
                tracing::error!("No found current user claims uname.");
                None
            }
        }
    }

    pub async fn get_current_email(&self) -> Option<String> {
        match self.get().await {
            Some(claims) => Some(claims.email),
            None => {
                tracing::error!("No found current user claims email.");
                None
            }
        }
    }

    pub async fn clear(&self) {
        let mut write_guard = self.current_user.write().await;
        *write_guard = None;
    }
}
