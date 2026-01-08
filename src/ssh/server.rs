use sqlx::PgPool;

pub async fn run_ssh_server(addr: String, _db: PgPool) -> crate::Result<()> {
    tracing::info!("SSH server will listen on {} (full implementation coming next)", addr);
    
    std::future::pending::<()>().await;
    Ok(())
}
