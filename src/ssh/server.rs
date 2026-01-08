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
    peer_addr: Option<std::net::SocketAddr>,
    id: usize,
}

impl Server {
    fn new(db: PgPool) -> Self {
        Self {
            db,
            clients: Arc::new(Mutex::new(HashMap::new())),
            apps: Arc::new(Mutex::new(HashMap::new())),
            peer_addr: None,
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
        .map_err(|e| russh::Error::from(std::io::Error::other(e.to_string())))?;

        let mut apps = self.apps.lock().await;
        if let Some(app) = apps.get_mut(&client_id) {
            app.set_posts(posts);
        }

        Ok(())
    }

    async fn verify_login(&self, username: &str, password: &str) -> Result<bool, russh::Error> {
        use crate::auth::AuthService;
        use crate::models::User;

        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Database error during login: {}", e);
                russh::Error::from(std::io::Error::other(e.to_string()))
            })?;

        match user {
            Some(user) => {
                let valid =
                    AuthService::verify_password(password, &user.password_hash).map_err(|e| {
                        tracing::error!("Password verification error: {}", e);
                        russh::Error::from(std::io::Error::other(e.to_string()))
                    })?;
                Ok(valid)
            }
            None => Ok(false),
        }
    }

    async fn render_client(&self, client_id: usize) -> Result<(), russh::Error> {
        let mut clients = self.clients.lock().await;
        let apps = self.apps.lock().await;

        if let (Some(terminal), Some(app)) = (clients.get_mut(&client_id), apps.get(&client_id)) {
            terminal
                .draw(|f| ui::render(f, app))
                .map_err(russh::Error::from)?;
        }

        Ok(())
    }
}

impl server::Server for Server {
    type Handler = Self;

    fn new_client(&mut self, peer_addr: Option<std::net::SocketAddr>) -> Self {
        let mut s = self.clone();
        s.peer_addr = peer_addr;
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

        // Extract just the base64-encoded key data (without algorithm prefix or comment)
        let key_str = key.to_string();
        let key_parts: Vec<&str> = key_str.split_whitespace().collect();
        let key_data = if key_parts.len() >= 2 {
            key_parts[1] // The base64 part
        } else {
            &key_str // Fallback to full string if format is unexpected
        };
        let key_type = key.algorithm().to_string();

        tracing::debug!(
            "Auth attempt: user={}, key_type={}, key_preview={}...",
            user,
            key_type,
            &key_data[..key_data.len().min(50)]
        );

        let authorized = sqlx::query_as::<_, AuthorizedKey>(
            "SELECT ak.* FROM authorized_keys ak
             JOIN users u ON ak.user_id = u.id
             WHERE u.username = $1 AND ak.public_key = $2 AND ak.key_type = $3",
        )
        .bind(user)
        .bind(key_data)
        .bind(&key_type)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking SSH key: {}", e);
            russh::Error::from(std::io::Error::other(e.to_string()))
        })?;

        if authorized.is_some() {
            tracing::info!("SSH authentication successful for user: {}", user);

            let mut apps = self.apps.lock().await;
            if let Some(app) = apps.get_mut(&self.id) {
                app.transition_to_browsing();
            }

            Ok(server::Auth::Accept)
        } else {
            tracing::warn!("SSH authentication failed for user: {}", user);
            Ok(server::Auth::Reject {
                proceed_with_methods: None,
                partial_success: false,
            })
        }
    }

    async fn auth_none(&mut self, user: &str) -> Result<server::Auth, Self::Error> {
        tracing::debug!("Auth none attempt for user: {}", user);

        if user == "bbs" {
            tracing::info!("Guest login accepted for user: bbs");
            Ok(server::Auth::Accept)
        } else {
            tracing::debug!("Auth none rejected for user: {}", user);
            Ok(server::Auth::Reject {
                proceed_with_methods: Some(MethodSet::from(&[MethodKind::PublicKey][..])),
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

        let apps = self.apps.lock().await;
        let is_browsing = apps
            .get(&self.id)
            .map(|app| matches!(app.state, ui::AppState::Browsing))
            .unwrap_or(false);
        drop(apps);

        if is_browsing {
            self.refresh_posts(self.id).await?;
        }
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
        let apps = self.apps.lock().await;
        let app_state = apps.get(&self.id).map(|app| app.state.clone());
        drop(apps);

        match app_state {
            Some(ui::AppState::Login) => {
                self.handle_login_input(data).await?;
                self.render_client(self.id).await?;
            }
            Some(ui::AppState::SecurityAlert) => {
                self.handle_alert_input(data).await?;
                self.render_client(self.id).await?;
            }
            Some(ui::AppState::Browsing) => {
                self.handle_browsing_input(channel, data, session).await?;
            }
            None => {}
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

impl Server {
    async fn handle_login_input(&mut self, data: &[u8]) -> Result<(), russh::Error> {
        let mut apps = self.apps.lock().await;
        let app = match apps.get_mut(&self.id) {
            Some(app) => app,
            None => return Ok(()),
        };

        match data {
            b"\r" | b"\n" => match app.login_step {
                ui::LoginStep::Username => {
                    if !app.input_buffer.is_empty() {
                        app.temp_username = Some(app.input_buffer.clone());
                        app.login_step = ui::LoginStep::Password;
                        app.clear_input();
                    }
                }
                ui::LoginStep::Password => {
                    let username = app.temp_username.clone().unwrap_or_default();
                    let password = app.input_buffer.clone();

                    drop(apps);

                    let valid = self.verify_login(&username, &password).await?;

                    let mut apps = self.apps.lock().await;
                    if let Some(app) = apps.get_mut(&self.id) {
                        if valid {
                            tracing::info!("Login successful for user: {}", username);

                            let current_ip = self
                                .peer_addr
                                .map(|addr| addr.ip().to_string())
                                .unwrap_or_else(|| "unknown".to_string());

                            drop(apps);

                            let user = sqlx::query_as::<_, crate::models::User>(
                                "SELECT * FROM users WHERE username = $1",
                            )
                            .bind(&username)
                            .fetch_one(&self.db)
                            .await
                            .map_err(|e| {
                                russh::Error::from(std::io::Error::other(e.to_string()))
                            })?;

                            let show_alert = match &user.last_login_ip {
                                Some(old_ip) if old_ip != &current_ip => true,
                                _ => false,
                            };

                            sqlx::query(
                                "UPDATE users SET last_login_ip = $1, last_login_at = NOW() WHERE username = $2",
                            )
                            .bind(&current_ip)
                            .bind(&username)
                            .execute(&self.db)
                            .await
                            .map_err(|e| russh::Error::from(std::io::Error::other(e.to_string())))?;

                            let mut apps = self.apps.lock().await;
                            if let Some(app) = apps.get_mut(&self.id) {
                                if show_alert {
                                    let old_ip =
                                        user.last_login_ip.unwrap_or_else(|| "unknown".to_string());
                                    app.show_security_alert(old_ip, current_ip);
                                } else {
                                    app.transition_to_browsing();
                                    drop(apps);
                                    self.refresh_posts(self.id).await?;
                                }
                            }
                        } else {
                            tracing::warn!("Login failed for user: {}", username);
                            app.reset_login(Some("Invalid username or password".to_string()));
                        }
                    }
                }
            },
            &[127] | b"\x08" => {
                app.backspace();
            }
            _ => {
                if data.len() == 1 && data[0].is_ascii_graphic() || data[0] == b' ' {
                    app.add_char(data[0] as char);
                }
            }
        }

        Ok(())
    }

    async fn handle_alert_input(&mut self, data: &[u8]) -> Result<(), russh::Error> {
        match data {
            b"\r" | b"\n" => {
                let mut apps = self.apps.lock().await;
                if let Some(app) = apps.get_mut(&self.id) {
                    app.transition_to_browsing();
                }
                drop(apps);
                self.refresh_posts(self.id).await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_browsing_input(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), russh::Error> {
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
        methods: MethodSet::from(&[MethodKind::PublicKey, MethodKind::None][..]),
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
