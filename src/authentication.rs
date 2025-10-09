use axum::{
    extract::Request,
    http::{HeaderMap, HeaderValue, header},
    middleware,
    response::IntoResponse,
};
use base64::Engine;
use reqwest::StatusCode;
use std::env;
use std::sync::LazyLock;

static LOGIN: LazyLock<Option<String>> = LazyLock::new(|| env::var("LOGIN").ok());
static PASSWORD: LazyLock<Option<String>> = LazyLock::new(|| env::var("PASSWORD").ok());

fn access_denied_response() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        "WWW-Authenticate",
        HeaderValue::from_static("Basic realm=\"Bandwidth-Hero Compression Service\""),
    );
    (
        StatusCode::UNAUTHORIZED,
        headers,
        "Access denied".to_string(),
    )
}

pub async fn authenticate(
    request: Request,
    next: middleware::Next,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let Some((expected_username, expected_password)) = LOGIN.clone().zip((*PASSWORD).clone())
    else {
        return Ok(next.run(request).await);
    };

    let given_credentials = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Basic "))
        .and_then(|h| base64::engine::general_purpose::STANDARD.decode(h).ok())
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .and_then(|s| {
            let mut parts = s.splitn(2, ':');
            Some((parts.next()?.to_string(), parts.next()?.to_string()))
        });

    match given_credentials {
        Some((ref user, ref pass)) if user == &expected_username && pass == &expected_password => {
            tracing::info!("Authentication successful for user: {}", user);
            Ok(next.run(request).await)
        }
        _ => {
            tracing::warn!("Authentication failed: {:?}", given_credentials);
            Err(access_denied_response())
        }
    }
}
