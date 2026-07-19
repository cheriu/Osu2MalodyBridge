use std::sync::Arc;

use axum::{routing::get, Router};

use crate::handlers;
use crate::services::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/store/info", get(handlers::server_info))
        .route("/api/store/list", get(handlers::song_list))
        .route("/api/store/promote", get(handlers::song_promote))
        .route("/api/store/charts", get(handlers::charts_list))
        .route("/api/store/download", get(handlers::download_chart))
        .route("/api/store/{cid}", get(handlers::send_chart_resource))
        .with_state(state)
}
