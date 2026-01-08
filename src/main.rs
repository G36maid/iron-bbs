use rusty_bbs::{Config, Result};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
    let ssh_handle = tokio::spawn(rusty_bbs::ssh::serve(config.ssh_addr(), db_pool));

    tokio::select! {
        result = web_handle => {
            tracing::error!("Web server stopped: {:?}", result);
        }
        result = ssh_handle => {
            tracing::error!("SSH server stopped: {:?}", result);
        }
    }

    Ok(())
}
