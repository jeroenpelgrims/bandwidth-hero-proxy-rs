use std::{collections::HashSet, sync::Arc};

use axum::{
    extract::State,
    http::HeaderMap,
    response::{AppendHeaders, IntoResponse},
};
use reqwest::{Client, header::HeaderValue};
use tracing;

use crate::{compress, error::AppError, headers, params::BandwidthHeroParams};

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

    tracing::info!("Proxying URL: {}", url);

    let mut headers = headers::filter_headers(headers, vec!["cookie", "dnt", "referer"]);
    headers.append("via", HeaderValue::from_static("1.1 bandwidth-hero"));
    headers.append(
        "user-agent",
        HeaderValue::from_static("Bandwidth-Hero Compressor"),
    );
    if let Some(forwarded_for) = headers.get("x-forwarded-for") {
        headers.append(
            "x-forwarded-for",
            reqwest::header::HeaderValue::from_bytes(forwarded_for.as_bytes())?,
        );
    }

    let response = client.get(url).headers(headers).send().await?;

    let buffer = response.bytes().await?;
    let original_size = buffer.len();
    let converted = compress::compress_image(buffer, quality.into(), monochrome, webp)?;
    let content_length = converted.len();
    let bytes_saved = original_size - content_length;
    let content_type = if webp { "image/webp" } else { "image/jpeg" };

    let mut response_headers = HeaderMap::new();
    response_headers.insert("content-encoding", HeaderValue::from_static("identity"));
    response_headers.insert("content-type", HeaderValue::from_static(content_type));
    response_headers.insert("content-length", HeaderValue::from(content_length));
    response_headers.insert("x-original-size", HeaderValue::from(original_size));
    response_headers.insert("x-bytes-saved", HeaderValue::from(bytes_saved));

    Ok((response_headers, converted).into_response())
}

async fn proxy_remote() {}
