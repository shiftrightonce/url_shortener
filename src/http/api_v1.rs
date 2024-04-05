use std::sync::Arc;

use axum::{
    extract::State, middleware, response::IntoResponse, routing::post, Extension, Json, Router,
};

use crate::{
    model::{api::ApiModel, url::UrlModel},
    AppState,
};

use super::{auth_middleware, ApiResponse};

pub(crate) fn setup_routes(app: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/generate", post(generate))
        .layer(middleware::from_fn_with_state(
            app.db_pool(),
            auth_middleware,
        ))
}

async fn generate(
    Extension(api): Extension<ApiModel>,
    State(app): State<Arc<AppState>>,
    Json(payload): Json<GeneratePayload>,
) -> impl IntoResponse {
    let pool = app.db_pool();

    let raw_url = payload.raw;
    let expires = payload.expires;

    // invalid
    if raw_url.is_empty() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            ApiResponse::<()>::error("raw is empty", "raw value must be a full URL"),
        )
            .into_response();
    }

    if expires > 0 && expires < chrono::Utc::now().timestamp_millis() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            ApiResponse::<()>::error(
                "wrong timestamp",
                "The timestamp must be in the future and in milliseconds",
            ),
        )
            .into_response();
    }

    let url = UrlModel::new(&raw_url, expires);
    if let Some(url) = url.save(&pool).await {
        return ApiResponse::<GenerateResponse>::succes(GenerateResponse {
            url: format!("{}/{}", api.domain().trim_end_matches('/'), url.short()),
            id: url.id(),
            expires: url.expires(),
        })
        .into_response();
    }

    (
        axum::http::StatusCode::BAD_REQUEST,
        ApiResponse::<()>::error("request was not handled", "Please check your payload"),
    )
        .into_response()
}

#[derive(Debug, Default, Clone, serde::Deserialize)]
struct GeneratePayload {
    raw: String,
    expires: i64,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
struct GenerateResponse {
    id: String,
    url: String,
    expires: i64,
}
