use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub author_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published: bool,
}

impl Post {
    pub fn preview(&self, length: usize) -> String {
        if self.content.len() <= length {
            self.content.clone()
        } else {
            format!("{}...", &self.content[..length])
        }
    }
}
