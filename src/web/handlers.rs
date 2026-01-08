use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json,
};
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
