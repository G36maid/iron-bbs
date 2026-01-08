use rand_core::OsRng;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::{Terminal, TerminalOptions, Viewport};
use russh::keys::*;
use russh::server::{Msg, Server as _, Session};
use russh::*;
use sqlx::PgPool;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use super::terminal::TerminalHandle;
use super::ui;

type SshTerminal = Terminal<CrosstermBackend<TerminalHandle>>;

#[derive(Clone)]
struct Server {
    db: PgPool,
    clients: Arc<Mutex<HashMap<usize, SshTerminal>>>,
    apps: Arc<Mutex<HashMap<usize, ui::App>>>,
    id: usize,
}

impl Server {
    fn new(db: PgPool) -> Self {
        Self {
            db,
            clients: Arc::new(Mutex::new(HashMap::new())),
            apps: Arc::new(Mutex::new(HashMap::new())),
            id: 0,
        }
    }

    async fn refresh_posts(&self, client_id: usize) -> Result<(), russh::Error> {
        use crate::models::Post;

        let posts = sqlx::query_as::<_, Post>(
            "SELECT * FROM posts WHERE published = true ORDER BY created_at DESC LIMIT 50",
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| {
            russh::Error::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        let mut apps = self.apps.lock().await;
        if let Some(app) = apps.get_mut(&client_id) {
            app.set_posts(posts);
        }

        Ok(())
    }

    async fn render_client(&self, client_id: usize) -> Result<(), russh::Error> {
        let mut clients = self.clients.lock().await;
        let apps = self.apps.lock().await;

        if let (Some(terminal), Some(app)) = (clients.get_mut(&client_id), apps.get(&client_id)) {
            terminal
                .draw(|f| ui::render(f, app))
                .map_err(|e| russh::Error::from(e))?;
        }

        Ok(())
    }
}

impl server::Server for Server {
    type Handler = Self;

    fn new_client(&mut self, _peer_addr: Option<std::net::SocketAddr>) -> Self {
        let mut s = self.clone();
        s.id = self.id + 1;
        self.id += 1;
        s
    }

    fn handle_session_error(&mut self, error: <Self::Handler as server::Handler>::Error) {
        tracing::error!("SSH session error: {:#?}", error);
    }
}

impl server::Handler for Server {
    type Error = russh::Error;

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        session: &mut Session,
    ) -> Result<bool, Self::Error> {
        let terminal_handle = TerminalHandle::start(session.handle(), channel.id()).await;
        let backend = CrosstermBackend::new(terminal_handle);

        let options = TerminalOptions {
            viewport: Viewport::Fixed(Rect::default()),
        };

        let terminal = Terminal::with_options(backend, options)?;
        let app = ui::App::new();

        self.clients.lock().await.insert(self.id, terminal);
        self.apps.lock().await.insert(self.id, app);

        Ok(true)
    }

    async fn auth_publickey(
        &mut self,
        user: &str,
        key: &ssh_key::PublicKey,
    ) -> Result<server::Auth, Self::Error> {
        use crate::models::AuthorizedKey;

        let key_str = key.to_string();
        let key_type = key.algorithm().to_string();

        let authorized = sqlx::query_as::<_, AuthorizedKey>(
            "SELECT ak.* FROM authorized_keys ak
             JOIN users u ON ak.user_id = u.id
             WHERE u.username = $1 AND ak.public_key = $2 AND ak.key_type = $3",
        )
        .bind(user)
        .bind(&key_str)
        .bind(&key_type)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking SSH key: {}", e);
            russh::Error::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        if authorized.is_some() {
            tracing::info!("SSH authentication successful for user: {}", user);
            Ok(server::Auth::Accept)
        } else {
            tracing::warn!("SSH authentication failed for user: {}", user);
            Ok(server::Auth::Reject {
                proceed_with_methods: None,
                partial_success: false,
            })
        }
    }

    async fn pty_request(
        &mut self,
        channel: ChannelId,
        _term: &str,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        _modes: &[(Pty, u32)],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let rect = Rect {
            x: 0,
            y: 0,
            width: col_width as u16,
            height: row_height as u16,
        };

        let mut clients = self.clients.lock().await;
        if let Some(terminal) = clients.get_mut(&self.id) {
            terminal.resize(rect)?;
        }

        session.channel_success(channel)?;

        drop(clients);

        self.refresh_posts(self.id).await?;
        self.render_client(self.id).await?;

        Ok(())
    }

    async fn window_change_request(
        &mut self,
        _channel: ChannelId,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        let rect = Rect {
            x: 0,
            y: 0,
            width: col_width as u16,
            height: row_height as u16,
        };

        let mut clients = self.clients.lock().await;
        if let Some(terminal) = clients.get_mut(&self.id) {
            terminal.resize(rect)?;
        }
        drop(clients);

        self.render_client(self.id).await?;

        Ok(())
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        match data {
            b"q" | &[3] => {
                self.clients.lock().await.remove(&self.id);
                self.apps.lock().await.remove(&self.id);
                session.close(channel)?;
            }
            b"k" | b"\x1b[A" => {
                let mut apps = self.apps.lock().await;
                if let Some(app) = apps.get_mut(&self.id) {
                    app.previous();
                }
                drop(apps);
                self.render_client(self.id).await?;
            }
            b"j" | b"\x1b[B" => {
                let mut apps = self.apps.lock().await;
                if let Some(app) = apps.get_mut(&self.id) {
                    app.next();
                }
                drop(apps);
                self.render_client(self.id).await?;
            }
            b"\r" | b"\n" => {
                let apps = self.apps.lock().await;
                let post_data = if let Some(app) = apps.get(&self.id) {
                    app.selected_post()
                        .map(|p| (p.title.clone(), p.content.clone()))
                } else {
                    None
                };
                drop(apps);

                if let Some((title, content)) = post_data {
                    let mut clients = self.clients.lock().await;
                    if let Some(terminal) = clients.get_mut(&self.id) {
                        let display = format!(
                            "\x1b[2J\x1b[H\r\n{}\r\n\r\n{}\r\n\r\nPress any key to return...",
                            title, content
                        );
                        Write::write_all(terminal.backend_mut(), display.as_bytes()).ok();
                        Write::flush(terminal.backend_mut()).ok();
                    }
                }
            }
            b"r" => {
                self.refresh_posts(self.id).await?;
                self.render_client(self.id).await?;
            }
            _ => {}
        }

        Ok(())
    }

    async fn shell_request(
        &mut self,
        _channel: ChannelId,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let id = self.id;
        let clients = self.clients.clone();
        let apps = self.apps.clone();
        tokio::spawn(async move {
            clients.lock().await.remove(&id);
            apps.lock().await.remove(&id);
        });
    }
}

pub async fn run_ssh_server(addr: String, db: PgPool) -> crate::Result<()> {
    let config = russh::server::Config {
        inactivity_timeout: Some(std::time::Duration::from_secs(3600)),
        auth_rejection_time: std::time::Duration::from_secs(3),
        auth_rejection_time_initial: Some(std::time::Duration::from_secs(0)),
        keys: vec![
            russh::keys::PrivateKey::random(&mut OsRng, russh::keys::Algorithm::Ed25519).map_err(
                |e| crate::Error::Internal(format!("Failed to generate SSH key: {}", e)),
            )?,
        ],
        ..Default::default()
    };

    let config = Arc::new(config);
    let mut server = Server::new(db);

    tracing::info!("SSH server listening on {} (TUI mode)", addr);

    let socket = TcpListener::bind(&addr).await?;
    server
        .run_on_socket(config, &socket)
        .await
        .map_err(|e| crate::Error::Internal(format!("SSH server error: {}", e)))?;

    Ok(())
}
