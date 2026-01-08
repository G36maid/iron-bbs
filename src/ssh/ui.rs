use crate::models::Post;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

pub struct App {
    pub posts: Vec<Post>,
    pub selected: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            posts: Vec::new(),
            selected: 0,
        }
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
