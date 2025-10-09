use axum::{extract::State, http::HeaderMap, response::IntoResponse};
use bytes::Bytes;
use reqwest::{Client, header::HeaderValue};
use std::sync::Arc;
use tracing;
use url::Url;

use crate::{compress, error::AppError, headers, params::BandwidthHeroParams};

pub async fn proxy_url(
    params: BandwidthHeroParams,
    headers: HeaderMap,
    State(client): State<Arc<Client>>,
) -> Result<impl IntoResponse, AppError> {
    let Some(url) = params.url.clone() else {
        return Ok("bandwidth-hero-proxy".into_response());
    };

    let original_image = proxy_remote(url, headers, &client).await?;
    let compressed_image = compress::compress_image(
        &original_image,
        params.quality.into(),
        params.monochrome,
        params.webp,
    )?;

    Ok(bwh_response(original_image, compressed_image, params).into_response())
}

async fn proxy_remote(
    url: Url,
    headers: HeaderMap,
    client: &Client,
) -> Result<bytes::Bytes, AppError> {
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
    Ok(response.bytes().await?)
}

fn bwh_response(
    original_image: Bytes,
    compressed_image: Bytes,
    params: BandwidthHeroParams,
) -> impl IntoResponse {
    let original_size = original_image.len();
    let content_length = compressed_image.len();
    let bytes_saved = original_size - content_length;
    let content_type = if params.webp {
        "image/webp"
    } else {
        "image/jpeg"
    };

    let mut response_headers = HeaderMap::new();
    response_headers.insert("content-encoding", HeaderValue::from_static("identity"));
    response_headers.insert("content-type", HeaderValue::from_static(content_type));
    response_headers.insert("content-length", HeaderValue::from(content_length));
    response_headers.insert("x-original-size", HeaderValue::from(original_size));
    response_headers.insert("x-bytes-saved", HeaderValue::from(bytes_saved));

    (response_headers, compressed_image)
}
