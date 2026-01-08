use rusty_bbs::{Config, Result};
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM signal");
        },
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rusty_bbs=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;

    tracing::info!("Starting rusty-bbs");
    tracing::info!("Web server will listen on: {}", config.web_addr());
    tracing::info!("SSH server will listen on: {}", config.ssh_addr());

    let db_pool = rusty_bbs::db::create_pool(&config.database_url).await?;

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    let app_state = rusty_bbs::web::AppState::new(db_pool.clone());

    let web_handle = tokio::spawn(rusty_bbs::web::serve(config.web_addr(), app_state));
    let ssh_handle = tokio::spawn(rusty_bbs::ssh::serve(config.ssh_addr(), db_pool.clone()));

    tokio::select! {
        result = web_handle => {
            tracing::error!("Web server exited: {:?}", result);
        }
        result = ssh_handle => {
            tracing::error!("SSH server exited: {:?}", result);
        }
        _ = shutdown_signal() => {
            tracing::info!("Shutdown signal received, closing database connections...");
            db_pool.close().await;
            tracing::info!("Graceful shutdown complete");
        }
    }

    Ok(())
}
