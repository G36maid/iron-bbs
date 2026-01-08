use rand_core::OsRng;
use russh::keys::*;
use russh::server::{Msg, Server as _, Session};
use russh::*;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Clone)]
struct Server {
    db: PgPool,
}

impl Server {
    fn new(db: PgPool) -> Self {
        Self { db }
    }
}

impl server::Server for Server {
    type Handler = Self;

    fn new_client(&mut self, _peer_addr: Option<std::net::SocketAddr>) -> Self {
        self.clone()
    }

    fn handle_session_error(&mut self, error: <Self::Handler as server::Handler>::Error) {
        tracing::error!("SSH session error: {:#?}", error);
    }
}

impl server::Handler for Server {
    type Error = russh::Error;

    async fn channel_open_session(
        &mut self,
        _channel: Channel<Msg>,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }

    async fn auth_publickey(
        &mut self,
        _user: &str,
        _key: &ssh_key::PublicKey,
    ) -> Result<server::Auth, Self::Error> {
        Ok(server::Auth::Accept)
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        if data == [3] {
            return Err(russh::Error::Disconnect);
        }

        let input = String::from_utf8_lossy(data).trim().to_string();

        if input == "quit" || input == "exit" {
            let _ = session.data(channel, CryptoVec::from("Goodbye!\r\n".to_string()));
            let _ = session.close(channel);
            return Ok(());
        }

        let response = if input.starts_with("view ") {
            let id_str = input.trim_start_matches("view ").trim();
            match self.view_post(id_str).await {
                Ok(content) => content,
                Err(e) => format!("Error: {}\r\n\r\n", e),
            }
        } else if input.starts_with("delete ") {
            let id_str = input.trim_start_matches("delete ").trim();
            match self.delete_post(id_str).await {
                Ok(msg) => msg,
                Err(e) => format!("Error: {}\r\n\r\n", e),
            }
        } else {
            match input.as_str() {
                "list" => match self.list_posts().await {
                    Ok(posts) => posts,
                    Err(e) => {
                        tracing::error!("Failed to list posts: {}", e);
                        "Error listing posts\r\n".to_string()
                    }
                },
                "help" => "Available commands:\r\n  list - List recent posts\r\n  view <id> - View post details\r\n  delete <id> - Delete a post\r\n  help - Show this help\r\n  quit - Exit\r\n\r\n".to_string(),
                "" => {
                    let _ = session.data(channel, CryptoVec::from("> ".to_string()));
                    return Ok(());
                }
                _ => format!("Unknown command: '{}'\r\nType 'help' for available commands.\r\n\r\n", input),
            }
        };

        let _ = session.data(channel, CryptoVec::from(response));
        let _ = session.data(channel, CryptoVec::from("> ".to_string()));

        Ok(())
    }

    async fn shell_request(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let welcome = "\r\n╔════════════════════════════════════════════╗\r\n";
        let _ = session.data(channel, CryptoVec::from(welcome.to_string()));
        
        let title = "║   Welcome to Rusty BBS (SSH Interface)  ║\r\n";
        let _ = session.data(channel, CryptoVec::from(title.to_string()));
        
        let bottom = "╚════════════════════════════════════════════╝\r\n\r\n";
        let _ = session.data(channel, CryptoVec::from(bottom.to_string()));

        let menu = "Commands:\r\n  list - List recent posts\r\n  help - Show this help\r\n  quit - Exit\r\n\r\n> ";
        let _ = session.data(channel, CryptoVec::from(menu.to_string()));

        Ok(())
    }
}

impl Server {
    async fn list_posts(&self) -> Result<String, sqlx::Error> {
        use crate::models::Post;

        let posts = sqlx::query_as::<_, Post>(
            "SELECT * FROM posts WHERE published = true ORDER BY created_at DESC LIMIT 10",
        )
        .fetch_all(&self.db)
        .await?;

        if posts.is_empty() {
            return Ok("No posts available.\r\n\r\n".to_string());
        }

        let mut output = String::from("╔══════════════════════════════════════════════════════════════════════╗\r\n");
        output.push_str("║                          RECENT POSTS                                ║\r\n");
        output.push_str("╚══════════════════════════════════════════════════════════════════════╝\r\n\r\n");

        for (idx, post) in posts.iter().enumerate() {
            output.push_str(&format!(
                "{}. {}\r\n   ID: {}\r\n   {}\r\n\r\n",
                idx + 1,
                post.title,
                post.id,
                post.preview(80)
            ));
        }

        Ok(output)
    }

    async fn view_post(&self, id_str: &str) -> Result<String, String> {
        use crate::models::Post;

        let id = uuid::Uuid::parse_str(id_str)
            .map_err(|_| "Invalid UUID format".to_string())?;

        let post = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or_else(|| "Post not found".to_string())?;

        let mut output = String::from("╔══════════════════════════════════════════════════════════════════════╗\r\n");
        output.push_str(&format!("║ {:<68} ║\r\n", post.title));
        output.push_str("╚══════════════════════════════════════════════════════════════════════╝\r\n\r\n");
        output.push_str(&format!("ID: {}\r\n", post.id));
        output.push_str(&format!("Author: {}\r\n", post.author_id));
        output.push_str(&format!("Published: {}\r\n", if post.published { "Yes" } else { "No" }));
        output.push_str(&format!("Created: {}\r\n\r\n", post.created_at.format("%Y-%m-%d %H:%M:%S")));
        output.push_str(&post.content);
        output.push_str("\r\n\r\n");

        Ok(output)
    }

    async fn delete_post(&self, id_str: &str) -> Result<String, String> {
        let id = uuid::Uuid::parse_str(id_str)
            .map_err(|_| "Invalid UUID format".to_string())?;

        let result = sqlx::query("DELETE FROM posts WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        if result.rows_affected() == 0 {
            return Err("Post not found".to_string());
        }

        Ok(format!("Post {} deleted successfully.\r\n\r\n", id))
    }
}

pub async fn run_ssh_server(addr: String, db: PgPool) -> crate::Result<()> {
    let config = russh::server::Config {
        inactivity_timeout: Some(std::time::Duration::from_secs(3600)),
        auth_rejection_time: std::time::Duration::from_secs(3),
        auth_rejection_time_initial: Some(std::time::Duration::from_secs(0)),
        keys: vec![russh::keys::PrivateKey::random(&mut OsRng, russh::keys::Algorithm::Ed25519).map_err(
            |e| crate::Error::Internal(format!("Failed to generate SSH key: {}", e)),
        )?],
        ..Default::default()
    };

    let config = Arc::new(config);
    let mut server = Server::new(db);

    tracing::info!("SSH server listening on {}", addr);

    let socket = TcpListener::bind(&addr).await?;
    server
        .run_on_socket(config, &socket)
        .await
        .map_err(|e| crate::Error::Internal(format!("SSH server error: {}", e)))?;

    Ok(())
}
