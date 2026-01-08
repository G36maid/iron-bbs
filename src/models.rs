use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_login_ip: Option<String>,
    pub last_login_at: Option<DateTime<Utc>>,
}

impl User {
    pub fn gravatar_url(&self, size: u32) -> String {
        let email_hash = format!(
            "{:x}",
            md5::compute(self.email.trim().to_lowercase().as_bytes())
        );
        format!(
            "https://www.gravatar.com/avatar/{}?s={}&d=identicon",
            email_hash, size
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub author_id: Uuid,
    pub board_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published: bool,
}

impl Post {
    pub fn preview(&self, length: usize) -> String {
        let chars: String = self.content.chars().take(length).collect();
        if self.content.chars().count() > length {
            format!("{}...", chars)
        } else {
            chars
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostWithAuthor {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub author_id: Uuid,
    pub author_username: String,
    pub author_email: String,
    pub board_id: Option<Uuid>,
    pub board_name: Option<String>,
    pub board_slug: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published: bool,
}

impl PostWithAuthor {
    pub fn preview(&self, length: usize) -> String {
        let chars: String = self.content.chars().take(length).collect();
        if self.content.chars().count() > length {
            format!("{}...", chars)
        } else {
            chars
        }
    }

    pub fn author_gravatar(&self, size: u32) -> String {
        let email_hash = format!(
            "{:x}",
            md5::compute(self.author_email.trim().to_lowercase().as_bytes())
        );
        format!(
            "https://www.gravatar.com/avatar/{}?s={}&d=identicon",
            email_hash, size
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Board {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuthorizedKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub public_key: String,
    pub key_type: String,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}
