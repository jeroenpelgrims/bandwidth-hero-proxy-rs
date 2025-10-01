use std::{collections::HashSet, sync::Arc};

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

    info!("Proxying URL: {}", url);

    let mut headers = headers::filter_headers(headers, vec!["cookie", "dnt", "referer"]);
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
    headers.append("via", HeaderValue::from_static("1.1 bandwidth-hero"));

    let response = client.get(url).headers(headers).send().await?;

    let response_headers = response.headers().clone();
    let content_type = response_headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let buffer = response.bytes().await?;
    let converted = todo::convert_bytes_to_webp_with_alpha(buffer, 1.0)?;

    Ok(converted.into_response())
}
