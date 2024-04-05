use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::request::Parts,
    middleware::Next,
    response::{IntoResponse, Response},
    Json, RequestExt, RequestPartsExt, Router,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use crate::{model::api::ApiModel, AppState};

mod api_v1;
mod general;

pub(crate) fn setup_routers(app_state: Arc<AppState>) -> Router {
    let mut app = Router::new();
    app = app.nest("/api/v1", api_v1::setup_routes(app_state.clone()));
    app = general::setup_routes(app);

    app.with_state(app_state)
}

pub(super) async fn auth_middleware(
    State(pool): State<sqlx::SqlitePool>,
    mut request: Request,
    next: Next,
) -> Response {
    let mut parts: Parts = request.extract_parts().await.unwrap();
    if let Ok(TypedHeader(Authorization(bearer))) =
        parts.extract::<TypedHeader<Authorization<Bearer>>>().await
    {
        if let Some(api) = ApiModel::find_by_request_token(bearer.token(), &pool).await {
            request.extensions_mut().insert(api);
            return next.run(request).await;
        }
    }

    let response =
        ApiResponse::<()>::error("Access Forbidden", "You do not have the right credentials")
            .into_response();

    (axum::http::StatusCode::FORBIDDEN, response).into_response()
}

#[derive(Debug, serde::Serialize)]
pub(super) struct ApiErrorMessage {
    code: String,
    message: String,
}

impl ApiErrorMessage {
    pub(super) fn new(code: impl ToString, message: impl ToString) -> Self {
        let clean = code.to_string().replace(' ', "_").to_lowercase();
        Self {
            code: clean,
            message: message.to_string(),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub(super) struct ApiResponse<D: serde::Serialize> {
    success: bool,
    data: Option<D>,
    error: Option<ApiErrorMessage>,
}

impl<D: serde::Serialize> ApiResponse<D> {
    pub(super) fn error(code: impl ToString, message: impl ToString) -> Self {
        Self {
            success: false,
            error: Some(ApiErrorMessage::new(code, message)),
            data: None,
        }
    }

    pub(super) fn succes(data: D) -> Self {
        Self {
            success: true,
            error: None,
            data: Some(data),
        }
    }
}

impl<D: serde::Serialize> IntoResponse for ApiResponse<D> {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}
