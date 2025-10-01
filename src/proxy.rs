use std::sync::Arc;

use axum::{extract::State, http::HeaderMap, response::IntoResponse};
use reqwest::{Client, header::HeaderValue};
use tracing::info;

use crate::{error::AppError, headers, params::BandwidthHeroParams, todo};

pub async fn proxy_url(
    BandwidthHeroParams {
        url,
        webp,
        monochrome,
        quality,
    }: BandwidthHeroParams,
    headers: HeaderMap,
    State(client): State<Arc<Client>>,
) -> Result<impl IntoResponse, AppError> {
    let Some(url) = url else {
        return Ok("bandwidth-hero-proxy".into_response());
    };

    let foo: axum::http::HeaderMap;
    let bar: reqwest::header::HeaderMap;

    info!("Proxying URL: {}", url);

    let mut request_headers = reqwest::header::HeaderMap::new();
    headers::forward_specific_headers(
        &headers,
        &mut request_headers,
        &["cookie", "dnt", "referer"],
    )?;

    request_headers.insert(
        "user-agent",
        HeaderValue::from_static("Bandwidth-Hero Compressor"),
    );

    if let Some(forwarded_for) = headers.get("x-forwarded-for") {
        request_headers.insert(
            "x-forwarded-for",
            reqwest::header::HeaderValue::from_bytes(forwarded_for.as_bytes())?,
        );
    }

    request_headers.insert("via", HeaderValue::from_static("1.1 bandwidth-hero"));

    let response = client.get(url).headers(request_headers).send().await?;

    let response_headers = response.headers().clone();
    let content_type = response_headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let buffer = response.bytes().await?;

    let mut final_headers = headers::reqwest_to_axum_headers(&response_headers)?;
    final_headers.insert(
        "content-encoding",
        axum::http::HeaderValue::from_static("identity"),
    );
    final_headers.insert(
        "content-type",
        axum::http::HeaderValue::from_static("image/webp"),
    );
    let converted = todo::convert_bytes_to_webp_with_alpha(buffer, 1.0)?;

    Ok(converted.into_response())
}
