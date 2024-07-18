use axum::{
    extract::{ Json, Query, State },
    http::StatusCode,
    response::IntoResponse,
    routing::{ get, post },
    Router,
};

use crate::{
    context::state::AppState,
    handlers::settings::ISettingsHandler,
    types::{
        settings::{ DeleteSettingsResponse, QuerySettingsResponse, SaveSettingsResponse },
        PageRequest,
    },
    utils::auths::SecurityContext,
};
use crate::handlers::settings::SettingsHandler;
use crate::types::settings::{ QuerySettingsRequest, SaveSettingsRequest, DeleteSettingsRequest };

use super::ValidatedJson;

pub fn init() -> Router<AppState> {
    Router::new()
        .route("/modules/settings/query", get(handle_get_settings))
        .route("/modules/settings/save", post(handle_save_settings))
        .route("/modules/settings/delete", post(handle_delete_settings))
}

#[utoipa::path(
    get,
    path = "/modules/settings/query",
    params(QuerySettingsRequest, PageRequest),
    responses((
        status = 200,
        description = "Getting for all settings.",
        body = QuerySettingsResponse,
    )),
    tag = "Settings"
)]
pub async fn handle_get_settings(
    State(state): State<AppState>,
    Query(param): Query<QuerySettingsRequest>,
    Query(page): Query<PageRequest>
) -> impl IntoResponse {
    let cur_settings = SecurityContext::get_instance().get().await;
    tracing::info!("current settings: {:?}", cur_settings);

    match get_settings_handler(&state).find(param, page).await {
        Ok((page, data)) => Ok(Json(QuerySettingsResponse::new(page, data))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    post,
    path = "/modules/settings/save",
    request_body = SaveSettingsRequest,
    responses((status = 200, description = "Save for settings.", body = SaveSettingsResponse)),
    tag = "Settings"
)]
async fn handle_save_settings(
    State(state): State<AppState>,
    ValidatedJson(param): ValidatedJson<SaveSettingsRequest>
) -> impl IntoResponse {
    match get_settings_handler(&state).save(param).await {
        Ok(result) => Ok(Json(SaveSettingsResponse::new(result))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    post,
    path = "/modules/settings/delete",
    request_body = DeleteSettingsRequest,
    responses((status = 200, description = "Delete for settings.", body = DeleteSettingsResponse)),
    tag = "Settings"
)]
async fn handle_delete_settings(
    State(state): State<AppState>,
    Json(param): Json<DeleteSettingsRequest>
) -> impl IntoResponse {
    match get_settings_handler(&state).delete(param).await {
        Ok(result) => Ok(Json(DeleteSettingsResponse::new(result))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

fn get_settings_handler(state: &AppState) -> Box<dyn ISettingsHandler + '_> {
    Box::new(SettingsHandler::new(state))
}
