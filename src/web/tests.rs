#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use serde_json::json;
    use sqlx::PgPool;
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{auth::AuthService, models::User, web::AppState};

    async fn setup_test_db() -> PgPool {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://iron_bbs:iron_bbs@localhost:5432/iron_bbs".to_string()
        });

        sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    async fn create_test_user(db: &PgPool) -> User {
        let username = format!("testuser_{}", Uuid::new_v4());
        let email = format!("{}@test.com", username);
        let password_hash = AuthService::hash_password("testpass123").unwrap();

        sqlx::query_as!(
            User,
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id, username, email, password_hash, created_at, last_login_ip, last_login_at",
            username,
            email,
            password_hash
        )
        .fetch_one(db)
        .await
        .expect("Failed to create test user")
    }

    async fn create_test_session(db: &PgPool, user_id: Uuid) -> String {
        let token = AuthService::generate_session_token();
        let expires_at = chrono::Utc::now() + chrono::Duration::days(7);

        sqlx::query!(
            "INSERT INTO sessions (user_id, token, expires_at) VALUES ($1, $2, $3)",
            user_id,
            token,
            expires_at
        )
        .execute(db)
        .await
        .expect("Failed to create test session");

        token
    }

    #[tokio::test]
    async fn test_create_post_without_auth() {
        let db = setup_test_db().await;
        let state = AppState::new(db.clone());
        let app = super::super::routes::create_routes().with_state(state);

        let user = create_test_user(&db).await;

        let payload = json!({
            "title": "Test Post",
            "content": "Test content",
            "author_id": user.id.to_string(),
            "published": true
        });

        let request = Request::builder()
            .method("POST")
            .uri("/api/posts")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Cleanup
        sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
            .execute(&db)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_create_post_with_auth() {
        let db = setup_test_db().await;
        let state = AppState::new(db.clone());
        let app = super::super::routes::create_routes().with_state(state);

        let user = create_test_user(&db).await;
        let token = create_test_session(&db, user.id).await;

        let payload = json!({
            "title": "Authenticated Test Post",
            "content": "Test content with auth",
            "author_id": user.id.to_string(),
            "published": true
        });

        let request = Request::builder()
            .method("POST")
            .uri("/api/posts")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", token))
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        // Cleanup
        sqlx::query!("DELETE FROM posts WHERE author_id = $1", user.id)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query!("DELETE FROM sessions WHERE token = $1", token)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
            .execute(&db)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_create_post_with_mismatched_author() {
        let db = setup_test_db().await;
        let state = AppState::new(db.clone());
        let app = super::super::routes::create_routes().with_state(state);

        let user1 = create_test_user(&db).await;
        let user2 = create_test_user(&db).await;
        let token = create_test_session(&db, user1.id).await;

        let payload = json!({
            "title": "Test Post",
            "content": "Test content",
            "author_id": user2.id.to_string(),
            "published": true
        });

        let request = Request::builder()
            .method("POST")
            .uri("/api/posts")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", token))
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Cleanup
        sqlx::query!("DELETE FROM sessions WHERE token = $1", token)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query!("DELETE FROM users WHERE id = $1", user1.id)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query!("DELETE FROM users WHERE id = $1", user2.id)
            .execute(&db)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_delete_post_without_auth() {
        let db = setup_test_db().await;
        let state = AppState::new(db.clone());
        let app = super::super::routes::create_routes().with_state(state);

        let user = create_test_user(&db).await;

        let post = sqlx::query!(
            "INSERT INTO posts (title, content, author_id, published) VALUES ($1, $2, $3, $4) RETURNING id",
            "Test Post",
            "Test content",
            user.id,
            true
        )
        .fetch_one(&db)
        .await
        .unwrap();

        let request = Request::builder()
            .method("DELETE")
            .uri(format!("/api/posts/{}", post.id))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Cleanup
        sqlx::query!("DELETE FROM posts WHERE id = $1", post.id)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
            .execute(&db)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_delete_post_by_non_author() {
        let db = setup_test_db().await;
        let state = AppState::new(db.clone());
        let app = super::super::routes::create_routes().with_state(state);

        let author = create_test_user(&db).await;
        let other_user = create_test_user(&db).await;
        let token = create_test_session(&db, other_user.id).await;

        let post = sqlx::query!(
            "INSERT INTO posts (title, content, author_id, published) VALUES ($1, $2, $3, $4) RETURNING id",
            "Test Post",
            "Test content",
            author.id,
            true
        )
        .fetch_one(&db)
        .await
        .unwrap();

        let request = Request::builder()
            .method("DELETE")
            .uri(format!("/api/posts/{}", post.id))
            .header("cookie", format!("session_id={}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Cleanup
        sqlx::query!("DELETE FROM posts WHERE id = $1", post.id)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query!("DELETE FROM sessions WHERE token = $1", token)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query!("DELETE FROM users WHERE id = $1", author.id)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query!("DELETE FROM users WHERE id = $1", other_user.id)
            .execute(&db)
            .await
            .unwrap();
    }
}
