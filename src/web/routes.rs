use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;

use super::{handlers, AppState};

pub fn create_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(handlers::index))
        .route("/posts/:id", get(handlers::get_post))
        .route("/api/posts", get(handlers::api_list_posts))
        .route("/health", get(handlers::health))
}
