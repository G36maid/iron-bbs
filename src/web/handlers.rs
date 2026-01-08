use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{models::Post, Error, Result};

use super::AppState;

pub async fn index(State(state): State<Arc<AppState>>) -> Result<Html<String>> {
    let posts = sqlx::query_as::<_, Post>(
        "SELECT * FROM posts WHERE published = true ORDER BY created_at DESC LIMIT 10",
    )
    .fetch_all(&state.db)
    .await?;

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Rusty BBS</title>
    <link href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css" rel="stylesheet">
</head>
<body class="bg-gray-100">
    <div class="container mx-auto px-4 py-8">
        <h1 class="text-4xl font-bold mb-8">Rusty BBS</h1>
        <div class="space-y-4">
            {}
        </div>
    </div>
</body>
</html>"#,
        posts
            .iter()
            .map(|p| format!(
                r#"<div class="bg-white p-6 rounded-lg shadow">
                    <h2 class="text-2xl font-semibold mb-2">{}</h2>
                    <p class="text-gray-600">{}</p>
                    <a href="/posts/{}" class="text-blue-500 hover:underline">Read more</a>
                </div>"#,
                p.title,
                p.preview(200),
                p.id
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );

    Ok(Html(html))
}

pub async fn get_post(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Html<String>> {
    let post = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = $1 AND published = true")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(Error::NotFound)?;

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>{} - Rusty BBS</title>
    <link href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css" rel="stylesheet">
</head>
<body class="bg-gray-100">
    <div class="container mx-auto px-4 py-8">
        <a href="/" class="text-blue-500 hover:underline mb-4 inline-block">&larr; Back</a>
        <article class="bg-white p-8 rounded-lg shadow">
            <h1 class="text-4xl font-bold mb-4">{}</h1>
            <div class="prose max-w-none">{}</div>
        </article>
    </div>
</body>
</html>"#,
        post.title, post.title, post.content
    );

    Ok(Html(html))
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
