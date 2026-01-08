use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use tower_cookies::Cookies;

const CSRF_HEADER: &str = "x-csrf-token";
const CSRF_COOKIE: &str = "csrf_token";

pub async fn csrf_protection(cookies: Cookies, request: Request, next: Next) -> Response {
    let method = request.method().clone();

    if method == "GET" || method == "HEAD" || method == "OPTIONS" {
        return next.run(request).await;
    }

    let cookie_token = cookies.get(CSRF_COOKIE).map(|c| c.value().to_string());
    let header_token = request
        .headers()
        .get(CSRF_HEADER)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    match (cookie_token, header_token) {
        (Some(cookie), Some(header)) if cookie == header => next.run(request).await,
        _ => {
            tracing::warn!("CSRF validation failed");
            Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body("CSRF validation failed".into())
                .unwrap()
        }
    }
}

pub fn verify_origin(request: &Request) -> bool {
    let origin = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|h| h.to_str().ok());

    let referer = request
        .headers()
        .get(header::REFERER)
        .and_then(|h| h.to_str().ok());

    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|h| h.to_str().ok());

    if let Some(host) = host {
        if let Some(origin) = origin {
            return origin.contains(host);
        }
        if let Some(referer) = referer {
            return referer.contains(host);
        }
    }

    false
}
