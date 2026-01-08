pub mod server;

use sqlx::PgPool;

pub async fn serve(addr: String, db: PgPool) -> crate::Result<()> {
    server::run_ssh_server(addr, db).await
}
