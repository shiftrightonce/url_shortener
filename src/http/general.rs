use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::Html,
    routing::get,
    Router,
};

use crate::{model::url::UrlModel, AppState};

pub(crate) fn setup_routes(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router.route("/:short", get(serve_url))
}

pub(crate) async fn serve_url(
    State(app): State<Arc<AppState>>,
    Path(short): Path<String>,
) -> Html<String> {
    let pool = app.db_pool();
    if let Some(url) = UrlModel::find_one_by_short(&short, &pool).await {
        Html(format!(
            "<head><meta http-equiv=\"Refresh\" content=\"0; URL={}\" /></head>",
            &url.raw()
        ))
    } else {
        Html(format!("Could not find url for: {}", short))
    }
}
