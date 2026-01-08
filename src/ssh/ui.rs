use crate::models::Post;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Login,
    SecurityAlert,
    Browsing,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoginStep {
    Username,
    Password,
}

pub struct App {
    pub state: AppState,
    pub login_step: LoginStep,
    pub input_buffer: String,
    pub temp_username: Option<String>,
    pub login_error: Option<String>,
    pub alert_info: Option<(String, String)>,
    pub posts: Vec<Post>,
    pub selected: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::Login,
            login_step: LoginStep::Username,
            input_buffer: String::new(),
            temp_username: None,
            login_error: None,
            alert_info: None,
            posts: Vec::new(),
            selected: 0,
        }
    }

    pub fn add_char(&mut self, c: char) {
        self.input_buffer.push(c);
    }

    pub fn backspace(&mut self) {
        self.input_buffer.pop();
    }

    pub fn clear_input(&mut self) {
        self.input_buffer.clear();
    }

    pub fn transition_to_browsing(&mut self) {
        self.state = AppState::Browsing;
        self.input_buffer.clear();
        self.temp_username = None;
        self.login_error = None;
    }

    pub fn reset_login(&mut self, error: Option<String>) {
        self.login_step = LoginStep::Username;
        self.input_buffer.clear();
        self.temp_username = None;
        self.login_error = error;
    }

    pub fn show_security_alert(&mut self, old_ip: String, new_ip: String) {
        self.state = AppState::SecurityAlert;
        self.alert_info = Some((old_ip, new_ip));
        self.input_buffer.clear();
        self.temp_username = None;
        self.login_error = None;
    }

    pub fn set_posts(&mut self, posts: Vec<Post>) {
        self.posts = posts;
        if self.selected >= self.posts.len() && !self.posts.is_empty() {
            self.selected = self.posts.len() - 1;
        }
    }

    pub fn next(&mut self) {
        if !self.posts.is_empty() {
            self.selected = (self.selected + 1) % self.posts.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.posts.is_empty() {
            self.selected = if self.selected == 0 {
                self.posts.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    pub fn selected_post(&self) -> Option<&Post> {
        self.posts.get(self.selected)
    }
}

pub fn render(f: &mut Frame, app: &App) {
    let area = f.size();

    match app.state {
        AppState::Login => render_login(f, app, area),
        AppState::SecurityAlert => render_security_alert(f, app, area),
        AppState::Browsing => render_browsing(f, app, area),
    }
}

fn render_login(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .margin(2)
        .split(area);

    let title = Paragraph::new("Welcome to Iron BBS")
        .block(Block::default().borders(Borders::ALL))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(title, chunks[0]);

    let (username_text, username_style) = match app.login_step {
        LoginStep::Username => {
            let text = if app.input_buffer.is_empty() {
                "Username: _".to_string()
            } else {
                format!("Username: {}_", app.input_buffer)
            };
            (text, Style::default().fg(Color::Yellow))
        }
        LoginStep::Password => {
            let username = app.temp_username.as_deref().unwrap_or("");
            (
                format!("Username: {}", username),
                Style::default().fg(Color::Gray),
            )
        }
    };

    let username_input = Paragraph::new(username_text)
        .block(Block::default().borders(Borders::ALL))
        .style(username_style);
    f.render_widget(username_input, chunks[1]);

    let (password_text, password_style) = match app.login_step {
        LoginStep::Username => ("Password: ".to_string(), Style::default().fg(Color::Gray)),
        LoginStep::Password => {
            let masked = "*".repeat(app.input_buffer.len());
            let text = if app.input_buffer.is_empty() {
                "Password: _".to_string()
            } else {
                format!("Password: {}_", masked)
            };
            (text, Style::default().fg(Color::Yellow))
        }
    };

    let password_input = Paragraph::new(password_text)
        .block(Block::default().borders(Borders::ALL))
        .style(password_style);
    f.render_widget(password_input, chunks[2]);

    if let Some(error) = &app.login_error {
        let error_msg = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(error_msg, chunks[3]);
    }
}

fn render_security_alert(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let default_ips = ("Unknown".to_string(), "Unknown".to_string());
    let (old_ip, new_ip) = app.alert_info.as_ref().unwrap_or(&default_ips);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title(" SECURITY ALERT ");

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let _inner = block.inner(chunks[0]);
    f.render_widget(block, area);

    let title_text = vec![Line::from(vec![Span::styled(
        "⚠️  SECURITY ALERT  ⚠️",
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    )])];
    let title = Paragraph::new(title_text).alignment(ratatui::layout::Alignment::Center);
    f.render_widget(title, chunks[0]);

    let message = Paragraph::new("Login detected from a different IP address!")
        .style(Style::default().fg(Color::Yellow))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(message, chunks[1]);

    let prev_ip_text = vec![Line::from(vec![
        Span::styled("Previous IP: ", Style::default().fg(Color::Gray)),
        Span::styled(old_ip, Style::default().fg(Color::Yellow)),
    ])];
    let prev_ip = Paragraph::new(prev_ip_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(ratatui::layout::Alignment::Left);
    f.render_widget(prev_ip, chunks[2]);

    let curr_ip_text = vec![Line::from(vec![
        Span::styled("Current IP:  ", Style::default().fg(Color::Gray)),
        Span::styled(new_ip, Style::default().fg(Color::Green)),
    ])];
    let curr_ip = Paragraph::new(curr_ip_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(ratatui::layout::Alignment::Left);
    f.render_widget(curr_ip, chunks[3]);

    let instruction = Paragraph::new("Press [Enter] to acknowledge and continue")
        .style(Style::default().fg(Color::Cyan))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(instruction, chunks[4]);
}

fn render_browsing(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    if app.posts.is_empty() {
        let paragraph = Paragraph::new("No posts available.\nPress 'q' to quit.")
            .block(Block::default().borders(Borders::ALL).title("Iron BBS"))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(paragraph, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let items: Vec<ListItem> = app
        .posts
        .iter()
        .enumerate()
        .map(|(idx, post)| {
            let is_selected = idx == app.selected;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let content = vec![
                Line::from(vec![
                    Span::styled(format!("{}. ", idx + 1), Style::default().fg(Color::Yellow)),
                    Span::styled(&post.title, style),
                ]),
                Line::from(Span::styled(
                    format!("   {}", post.preview(60)),
                    Style::default().fg(Color::Gray),
                )),
            ];
            ListItem::new(content)
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Iron BBS - Posts (Interactive TUI)"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(list, chunks[0], &mut list_state);

    let footer_text = vec![Line::from(vec![
        Span::styled("↑/k", Style::default().fg(Color::Yellow)),
        Span::raw(" up | "),
        Span::styled("↓/j", Style::default().fg(Color::Yellow)),
        Span::raw(" down | "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" view | "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(" quit"),
    ])];

    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    f.render_widget(footer, chunks[1]);
}
