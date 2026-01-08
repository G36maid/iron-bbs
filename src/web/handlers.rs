use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Form, Json,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;

use crate::{
    auth::AuthService,
    models::{Post, PostWithAuthor, User},
    Error, Result,
};

use super::{AppState, AuthPayload, CreatePostPayload, RegisterPayload};

async fn check_auth(cookies: &Cookies, db: &sqlx::PgPool) -> Option<User> {
    let session_cookie = cookies.get("session_id")?;
    let token = session_cookie.value();

    let session = sqlx::query!(
        "SELECT user_id FROM sessions WHERE token = $1 AND expires_at > NOW()",
        token
    )
    .fetch_optional(db)
    .await
    .ok()??;

    let user = sqlx::query_as!(
        User,
        "SELECT id, username, email, password_hash, created_at, last_login_ip, last_login_at FROM users WHERE id = $1",
        session.user_id
    )
    .fetch_optional(db)
    .await
    .ok()??;

    Some(user)
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    posts: Vec<PostWithAuthor>,
    current_user: Option<String>,
}

#[derive(Template)]
#[template(path = "post.html")]
struct PostTemplate {
    post: PostWithAuthor,
    author_gravatar: String,
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    error: Option<String>,
    current_user: Option<String>,
}

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate {
    error: Option<String>,
    current_user: Option<String>,
}

#[derive(Template)]
#[template(path = "create_post.html")]
struct CreatePostTemplate {
    error: Option<String>,
    current_user: Option<String>,
}

pub async fn index(State(state): State<Arc<AppState>>, cookies: Cookies) -> Result<Response> {
    let posts = sqlx::query_as!(
        PostWithAuthor,
        r#"
        SELECT 
            p.id, p.title, p.content, p.author_id, p.created_at, p.updated_at, p.published,
            p.board_id, b.name as board_name, b.slug as board_slug,
            u.username as author_username, u.email as author_email
        FROM posts p
        JOIN users u ON p.author_id = u.id
        LEFT JOIN boards b ON p.board_id = b.id
        WHERE p.published = true
        ORDER BY p.created_at DESC
        LIMIT 10
        "#
    )
    .fetch_all(&state.db)
    .await?;

    let current_user = check_auth(&cookies, &state.db).await.map(|u| u.username);

    let template = IndexTemplate {
        posts,
        current_user,
    };
    Ok(Html(
        template
            .render()
            .map_err(|e| Error::Internal(format!("Template error: {}", e)))?,
    )
    .into_response())
}

pub async fn get_post(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    let post = sqlx::query_as!(
        PostWithAuthor,
        r#"
        SELECT 
            p.id, p.title, p.content, p.author_id, p.created_at, p.updated_at, p.published,
            p.board_id, b.name as board_name, b.slug as board_slug,
            u.username as author_username, u.email as author_email
        FROM posts p
        JOIN users u ON p.author_id = u.id
        LEFT JOIN boards b ON p.board_id = b.id
        WHERE p.id = $1 AND p.published = true
        "#,
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(Error::NotFound)?;

    let author_gravatar = post.author_gravatar(64);

    let template = PostTemplate {
        post,
        author_gravatar,
    };
    Ok(Html(
        template
            .render()
            .map_err(|e| Error::Internal(format!("Template error: {}", e)))?,
    )
    .into_response())
}

pub async fn login_form(cookies: Cookies, State(state): State<Arc<AppState>>) -> Result<Response> {
    let current_user = check_auth(&cookies, &state.db).await.map(|u| u.username);

    let template = LoginTemplate {
        error: None,
        current_user,
    };
    Ok(Html(
        template
            .render()
            .map_err(|e| Error::Internal(format!("Template error: {}", e)))?,
    )
    .into_response())
}

pub async fn login_submit(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    Form(payload): Form<AuthPayload>,
) -> Result<Response> {
    let user =
        AuthService::authenticate_user(&state.db, &payload.username, &payload.password).await?;

    let user = match user {
        Some(u) => u,
        None => {
            let template = LoginTemplate {
                error: Some("Invalid username or password".to_string()),
                current_user: None,
            };
            return Ok(Html(
                template
                    .render()
                    .map_err(|e| Error::Internal(format!("Template error: {}", e)))?,
            )
            .into_response());
        }
    };

    let token = AuthService::generate_session_token();
    let expires_at = Utc::now() + Duration::days(7);

    sqlx::query!(
        "INSERT INTO sessions (user_id, token, expires_at) VALUES ($1, $2, $3)",
        user.id,
        token,
        expires_at
    )
    .execute(&state.db)
    .await?;

    let mut cookie = Cookie::new("session_id", token);
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookies.add(cookie);

    Ok(Redirect::to("/").into_response())
}

pub async fn register_form(
    cookies: Cookies,
    State(state): State<Arc<AppState>>,
) -> Result<Response> {
    let current_user = check_auth(&cookies, &state.db).await.map(|u| u.username);

    let template = RegisterTemplate {
        error: None,
        current_user,
    };
    Ok(Html(
        template
            .render()
            .map_err(|e| Error::Internal(format!("Template error: {}", e)))?,
    )
    .into_response())
}

pub async fn register_submit(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    Form(payload): Form<RegisterPayload>,
) -> Result<Response> {
    if payload.username.len() < 3 {
        let template = RegisterTemplate {
            error: Some("Username must be at least 3 characters".to_string()),
            current_user: None,
        };
        return Ok(Html(
            template
                .render()
                .map_err(|e| Error::Internal(format!("Template error: {}", e)))?,
        )
        .into_response());
    }

    if payload.password.len() < 8 {
        let template = RegisterTemplate {
            error: Some("Password must be at least 8 characters".to_string()),
            current_user: None,
        };
        return Ok(Html(
            template
                .render()
                .map_err(|e| Error::Internal(format!("Template error: {}", e)))?,
        )
        .into_response());
    }

    let existing_user = sqlx::query!(
        "SELECT id FROM users WHERE username = $1 OR email = $2",
        payload.username,
        payload.email
    )
    .fetch_optional(&state.db)
    .await?;

    if existing_user.is_some() {
        let template = RegisterTemplate {
            error: Some("Username or email already exists".to_string()),
            current_user: None,
        };
        return Ok(Html(
            template
                .render()
                .map_err(|e| Error::Internal(format!("Template error: {}", e)))?,
        )
        .into_response());
    }

    let password_hash = AuthService::hash_password(&payload.password)?;

    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id, username, email, password_hash, created_at, last_login_ip, last_login_at",
        payload.username,
        payload.email,
        password_hash
    )
    .fetch_one(&state.db)
    .await?;

    let token = AuthService::generate_session_token();
    let expires_at = Utc::now() + Duration::days(7);

    sqlx::query!(
        "INSERT INTO sessions (user_id, token, expires_at) VALUES ($1, $2, $3)",
        user.id,
        token,
        expires_at
    )
    .execute(&state.db)
    .await?;

    let mut cookie = Cookie::new("session_id", token);
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookies.add(cookie);

    Ok(Redirect::to("/").into_response())
}

pub async fn logout(cookies: Cookies, State(state): State<Arc<AppState>>) -> Result<Response> {
    if let Some(session_cookie) = cookies.get("session_id") {
        let token = session_cookie.value();
        sqlx::query!("DELETE FROM sessions WHERE token = $1", token)
            .execute(&state.db)
            .await?;
    }

    cookies.remove(Cookie::from("session_id"));
    Ok(Redirect::to("/").into_response())
}

pub async fn create_post_form(
    cookies: Cookies,
    State(state): State<Arc<AppState>>,
) -> Result<Response> {
    let current_user = check_auth(&cookies, &state.db).await;

    if current_user.is_none() {
        return Ok(Redirect::to("/login").into_response());
    }

    let template = CreatePostTemplate {
        error: None,
        current_user: current_user.map(|u| u.username),
    };
    Ok(Html(
        template
            .render()
            .map_err(|e| Error::Internal(format!("Template error: {}", e)))?,
    )
    .into_response())
}

pub async fn create_post_submit(
    cookies: Cookies,
    State(state): State<Arc<AppState>>,
    Form(payload): Form<CreatePostPayload>,
) -> Result<Response> {
    let user = check_auth(&cookies, &state.db).await;

    let user = match user {
        Some(u) => u,
        None => {
            return Ok(Redirect::to("/login").into_response());
        }
    };

    if payload.title.trim().is_empty() {
        let template = CreatePostTemplate {
            error: Some("Title cannot be empty".to_string()),
            current_user: Some(user.username),
        };
        return Ok(Html(
            template
                .render()
                .map_err(|e| Error::Internal(format!("Template error: {}", e)))?,
        )
        .into_response());
    }

    if payload.content.trim().is_empty() {
        let template = CreatePostTemplate {
            error: Some("Content cannot be empty".to_string()),
            current_user: Some(user.username),
        };
        return Ok(Html(
            template
                .render()
                .map_err(|e| Error::Internal(format!("Template error: {}", e)))?,
        )
        .into_response());
    }

    let published = payload.published.is_some();

    sqlx::query!(
        "INSERT INTO posts (title, content, author_id, published) VALUES ($1, $2, $3, $4)",
        payload.title,
        payload.content,
        user.id,
        published
    )
    .execute(&state.db)
    .await?;

    Ok(Redirect::to("/").into_response())
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
    cookies: Cookies,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreatePostRequest>,
) -> Result<(StatusCode, Json<Post>)> {
    let user = check_auth(&cookies, &state.db)
        .await
        .ok_or(Error::Unauthorized)?;

    if user.id != payload.author_id {
        return Err(Error::Unauthorized);
    }

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
    cookies: Cookies,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePostRequest>,
) -> Result<Json<Post>> {
    let user = check_auth(&cookies, &state.db)
        .await
        .ok_or(Error::Unauthorized)?;

    let existing_post = sqlx::query!("SELECT author_id FROM posts WHERE id = $1", id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(Error::NotFound)?;

    if user.id != existing_post.author_id {
        return Err(Error::Unauthorized);
    }

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

    let post = q.fetch_optional(&state.db).await?.ok_or(Error::NotFound)?;

    Ok(Json(post))
}

pub async fn delete_post(
    cookies: Cookies,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let user = check_auth(&cookies, &state.db)
        .await
        .ok_or(Error::Unauthorized)?;

    let existing_post = sqlx::query!("SELECT author_id FROM posts WHERE id = $1", id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(Error::NotFound)?;

    if user.id != existing_post.author_id {
        return Err(Error::Unauthorized);
    }

    let result = sqlx::query("DELETE FROM posts WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
