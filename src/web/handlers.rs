use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{models::Post, Error, Result};

use super::AppState;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    posts: Vec<Post>,
}

#[derive(Template)]
#[template(path = "post.html")]
struct PostTemplate {
    post: Post,
}

pub async fn index(State(state): State<Arc<AppState>>) -> Result<Response> {
    let posts = sqlx::query_as::<_, Post>(
        "SELECT * FROM posts WHERE published = true ORDER BY created_at DESC LIMIT 10",
    )
    .fetch_all(&state.db)
    .await?;

    let template = IndexTemplate { posts };
    Ok(Html(template.render().map_err(|e| {
        Error::Internal(format!("Template error: {}", e))
    })?).into_response())
}

pub async fn get_post(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    let post = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = $1 AND published = true")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(Error::NotFound)?;

    let template = PostTemplate { post };
    Ok(Html(template.render().map_err(|e| {
        Error::Internal(format!("Template error: {}", e))
    })?).into_response())
}

pub async fn api_list_posts(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Post>>> {
    let posts = sqlx::query_as::<_, Post>(
        "SELECT * FROM posts WHERE published = true ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(posts))
}

pub async fn health() -> (StatusCode, &'static str) {
    (StatusCode::OK, "OK")
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
    pub author_id: Uuid,
    pub published: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub published: Option<bool>,
}

pub async fn create_post(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreatePostRequest>,
) -> Result<(StatusCode, Json<Post>)> {
    let post = sqlx::query_as::<_, Post>(
        "INSERT INTO posts (title, content, author_id, published) VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(payload.author_id)
    .bind(payload.published.unwrap_or(false))
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(post)))
}

pub async fn update_post(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePostRequest>,
) -> Result<Json<Post>> {
    let mut query = String::from("UPDATE posts SET updated_at = NOW()");
    let mut bind_count = 1;

    if payload.title.is_some() {
        query.push_str(&format!(", title = ${}", bind_count));
        bind_count += 1;
    }
    if payload.content.is_some() {
        query.push_str(&format!(", content = ${}", bind_count));
        bind_count += 1;
    }
    if payload.published.is_some() {
        query.push_str(&format!(", published = ${}", bind_count));
        bind_count += 1;
    }

    query.push_str(&format!(" WHERE id = ${} RETURNING *", bind_count));

    let mut q = sqlx::query_as::<_, Post>(&query);

    if let Some(title) = &payload.title {
        q = q.bind(title);
    }
    if let Some(content) = &payload.content {
        q = q.bind(content);
    }
    if let Some(published) = payload.published {
        q = q.bind(published);
    }

    q = q.bind(id);

    let post = q.fetch_optional(&state.db)
        .await?
        .ok_or(Error::NotFound)?;

    Ok(Json(post))
}

pub async fn delete_post(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let result = sqlx::query("DELETE FROM posts WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
