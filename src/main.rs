use std::{sync::Arc, time::Duration};

use axum::{
    Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
};
use bytes::Bytes;
use error::AppError;
use params::{BandwidthHeroParams, RawBandwidthHeroParams};
use reqwest::{
    Client,
    header::{HeaderName, HeaderValue},
    redirect::Policy,
};
use webp::Encoder;

mod error;
mod headers;
mod params;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let http_client = Client::builder()
        .timeout(Duration::from_millis(10000))
        .redirect(Policy::limited(5))
        .danger_accept_invalid_certs(true)
        .gzip(true)
        .cookie_store(true)
        .build()?;

    let port = 8081;
    let app = Router::new()
        .route("/", get(proxy_url))
        .with_state(Arc::new(http_client));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn proxy_url(
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

    println!("Proxying URL: {}", url);

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
    let converted = convert_bytes_to_webp_with_alpha(buffer, 1.0)?;

    Ok(converted.into_response())
}

fn convert_bytes_to_webp_with_alpha(
    image_bytes: Bytes,
    quality: f32,
) -> Result<Bytes, anyhow::Error> {
    let img = image::load_from_memory(&image_bytes)?;

    let rgba_img = match img {
        image::DynamicImage::ImageRgba8(rgba) => rgba,
        other => other.to_rgba8(),
    };
    let (width, height) = rgba_img.dimensions();

    // Create encoder from RGBA data
    let encoder = Encoder::from_rgba(&rgba_img, width, height);
    let webp_data = encoder.encode(quality);

    Ok(Bytes::from(webp_data.to_vec()))
}
