mod handlers;
mod routes;

#[cfg(test)]
mod tests;

use axum::Router;
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

impl AppState {
    pub fn new(db: PgPool) -> Arc<Self> {
        Arc::new(Self { db })
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthPayload {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterPayload {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePostPayload {
    pub title: String,
    pub content: String,
    pub published: Option<String>,
}

pub async fn serve(addr: String, state: Arc<AppState>) -> crate::Result<()> {
    let app = Router::new()
        .merge(routes::create_routes())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Web server listening on {}", addr);

    axum::serve(listener, app)
        .await
        .map_err(|e| crate::Error::Internal(e.to_string()))?;

    Ok(())
}
