pub mod handlers;
pub mod routes;

use axum::Router;
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
