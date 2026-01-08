use axum::{
    routing::{get, post, put},
    Router,
};
use std::sync::Arc;
use tower_cookies::CookieManagerLayer;

use super::{handlers, AppState};

pub fn create_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(handlers::index))
        .route(
            "/login",
            get(handlers::login_form).post(handlers::login_submit),
        )
        .route(
            "/register",
            get(handlers::register_form).post(handlers::register_submit),
        )
        .route("/logout", post(handlers::logout))
        .route(
            "/new",
            get(handlers::create_post_form).post(handlers::create_post_submit),
        )
        .route("/posts/:id", get(handlers::get_post))
        .route(
            "/api/posts",
            get(handlers::api_list_posts).post(handlers::create_post),
        )
        .route(
            "/api/posts/:id",
            put(handlers::update_post).delete(handlers::delete_post),
        )
        .route("/health", get(handlers::health))
        .layer(CookieManagerLayer::new())
}
