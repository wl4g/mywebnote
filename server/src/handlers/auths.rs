use std::{ collections::HashMap, sync::Arc };

use axum::async_trait;
use hyper::{ header, StatusCode };
use lazy_static::lazy_static;
use anyhow::{ Error, Ok };
use chrono::Utc;
use openidconnect::{ core::CoreUserInfoClaims, LanguageTag };
use tower_cookies::cookie::{ time::Duration, CookieBuilder, SameSite };
use crate::{
    config::config_api::ApiConfig,
    context::state::AppState,
    types::{ auths::{ GithubUserInfo, LogoutRequest }, users::SaveUserRequest },
    utils::{ self, auths },
};

use super::users::{ IUserHandler, UserHandler };

pub const AUTH_NONCE_PREFIX: &'static str = "auth:nonce:";
pub const AUTH_LOGOUT_BLACKLIST_PREFIX: &'static str = "auth:logout:blacklist:";

lazy_static! {
    pub static ref LANG_CLAIMS_NAME_KEY: LanguageTag = LanguageTag::new("name".to_owned());
}

#[async_trait]
pub trait IAuthHandler: Send {
    async fn handle_auth_create_nonce(&self, sid: &str, nonce: String) -> Result<(), Error>;

    async fn handle_auth_get_nonce(&self, sid: &str) -> Result<Option<String>, Error>;

    async fn handle_auth_callback_oidc(&self, userinfo: CoreUserInfoClaims) -> Result<i64, Error>;

    async fn handle_auth_callback_github(&self, userinfo: GithubUserInfo) -> Result<i64, Error>;

    async fn handle_login_success(
        &self,
        config: &Arc<ApiConfig>,
        user_id: &str,
        headers: &header::HeaderMap
    ) -> hyper::Response<axum::body::Body>;

    async fn handle_logout(&self, param: LogoutRequest) -> Result<(), Error>;

    fn build_auth_nonce_key(&self, nonce: &str) -> String;

    fn build_logout_blacklist_key(&self, access_token: &str) -> String;
}

pub struct AuthHandler<'a> {
    state: &'a AppState,
}

impl<'a> AuthHandler<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl<'a> IAuthHandler for AuthHandler<'a> {
    async fn handle_auth_create_nonce(&self, sid: &str, nonce: String) -> Result<(), Error> {
        let cache = self.state.string_cache.cache(&self.state.config);

        let key = self.build_logout_blacklist_key(sid);
        let value = nonce;

        // TODO: using expires config? To ensure safety, expire as soon as possible. 10s
        match cache.set(key, value, Some(10_000)).await {
            std::result::Result::Ok(_) => {
                tracing::info!("Created auth nonce for {}", sid);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Created auth nonce failed for {}, cause: {}", sid, e);
                Err(e)
            }
        }
    }

    async fn handle_auth_get_nonce(&self, sid: &str) -> Result<Option<String>, Error> {
        let cache = self.state.string_cache.cache(&self.state.config);

        let key = self.build_logout_blacklist_key(sid);

        match cache.get(key).await {
            std::result::Result::Ok(nonce) => {
                tracing::info!("Got auth nonce for {}", sid);
                Ok(nonce)
            }
            Err(e) => {
                tracing::error!("Get auth nonce failed for {}, cause: {}", sid, e);
                Err(e)
            }
        }
    }

    async fn handle_auth_callback_oidc(&self, userinfo: CoreUserInfoClaims) -> Result<i64, Error> {
        let oidc_user_id = userinfo.subject().as_str();
        let oidc_user_name = userinfo.name().map(|n|
            n
                .get(Some(&LANG_CLAIMS_NAME_KEY))
                .map(|u| u.to_string())
                .unwrap_or_default()
        );

        let handler = UserHandler::new(self.state);

        // 1. Get user by oidc user_id
        let user = handler.get(Some(oidc_user_id.to_string()), None, None).await.unwrap();

        // 2. If user exists, update user github subject ID.
        let save_param;
        if user.is_some() {
            save_param = SaveUserRequest {
                id: user.unwrap().base.id,
                name: oidc_user_name.to_owned(),
                email: None,
                phone: None,
                password: None,
                oidc_claims_sub: Some(oidc_user_id.to_string()),
                oidc_claims_name: oidc_user_name,
                github_claims_sub: None,
                github_claims_name: None,
                google_claims_sub: None,
                google_claims_name: None,
            };
        } else {
            // 3. If user not exists, create user by github login, which auto register user.
            save_param = SaveUserRequest {
                id: None,
                name: oidc_user_name.to_owned(),
                email: None,
                phone: None,
                password: None,
                oidc_claims_sub: Some(oidc_user_id.to_string()),
                oidc_claims_name: oidc_user_name,
                github_claims_sub: None,
                github_claims_name: None,
                google_claims_sub: None,
                google_claims_name: None,
            };
        }

        handler.save(save_param).await
    }

    async fn handle_auth_callback_github(&self, userinfo: GithubUserInfo) -> Result<i64, Error> {
        let github_user_id = userinfo.id.expect("github user_id is None");
        let github_user_name = userinfo.login.expect("github user_name is None");

        let handler = UserHandler::new(self.state);

        // 1. Get user by github_user_id
        let user = handler.get(None, Some(github_user_id.to_string()), None).await.unwrap();

        // 2. If user exists, update user github subject ID.
        let save_param;
        if user.is_some() {
            save_param = SaveUserRequest {
                id: user.unwrap().base.id,
                name: Some(github_user_name.to_string()),
                email: None,
                phone: None,
                password: None,
                oidc_claims_sub: None,
                oidc_claims_name: None,
                github_claims_sub: Some(github_user_id.to_string()),
                github_claims_name: Some(github_user_name.to_string()),
                google_claims_sub: None,
                google_claims_name: None,
            };
        } else {
            // 3. If user not exists, create user by github login, which auto register user.
            save_param = SaveUserRequest {
                id: None,
                name: Some(github_user_name.to_string()),
                email: None,
                phone: None,
                password: None,
                oidc_claims_sub: None,
                oidc_claims_name: None,
                github_claims_sub: Some(github_user_id.to_string()),
                github_claims_name: Some(github_user_name.to_string()),
                google_claims_sub: None,
                google_claims_name: None,
            };
        }

        handler.save(save_param).await
    }

    async fn handle_login_success(
        &self,
        config: &Arc<ApiConfig>,
        user_id: &str,
        headers: &header::HeaderMap
    ) -> hyper::Response<axum::body::Body> {
        // TODO: 附加更多自定义 JWT 信息
        let extra_claims = HashMap::new();
        let ak = auths::create_jwt(config, user_id, false, Some(extra_claims));
        let rk = auths::create_jwt(config, user_id, true, None);

        let ak_cookie = CookieBuilder::new(&config.auth_jwt_ak_name, ak)
            .path("/")
            .max_age(Duration::milliseconds(config.auth.jwt_validity_ak.unwrap() as i64))
            .secure(true)
            .http_only(true)
            .same_site(SameSite::Strict)
            .build();

        let rk_cookie = CookieBuilder::new(&config.auth_jwt_rk_name, rk)
            .path("/")
            .max_age(Duration::milliseconds(config.auth.jwt_validity_rk.unwrap() as i64))
            .secure(true)
            .http_only(true)
            .same_site(SameSite::Strict)
            .build();

        // use axum::response::{ IntoResponse, Redirect };
        // let mut response = Redirect::to("/").into_response();
        // utils::auths::add_cookies(&mut response, vec![ak_cookie, rk_cookie]);
        // response

        utils::auths::auth_resp_redirect_or_json(
            &config,
            headers,
            config.auth.success_url.to_owned().unwrap().as_str(),
            StatusCode::OK,
            "Authenticated",
            Some((ak_cookie, rk_cookie))
        )
    }

    async fn handle_logout(&self, param: LogoutRequest) -> Result<(), Error> {
        let cache = self.state.string_cache.cache(&self.state.config);

        // Add current jwt token to cache blacklist, expiration time is less than now time - id_token issue time.
        let ak = match param.access_token {
            Some(v) => v.to_string(),
            None => {
                return Err(Error::msg("access_token is None"));
            }
        };
        let key = self.build_logout_blacklist_key(ak.as_str());
        let value = Utc::now().timestamp_millis().to_string();
        match cache.set(key, value, Some(3600_000)).await {
            std::result::Result::Ok(_) => {
                tracing::info!("Logout success for {}", ak);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Logout failed: {}, cause: {}", ak, e);
                Err(e)
            }
        }
    }

    fn build_auth_nonce_key(&self, nonce: &str) -> String {
        format!("{}:{}", AUTH_NONCE_PREFIX, nonce)
    }

    fn build_logout_blacklist_key(&self, access_token: &str) -> String {
        format!("{}:{}", AUTH_LOGOUT_BLACKLIST_PREFIX, access_token)
    }
}
