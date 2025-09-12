use axum::{Router, extract::Query, response::IntoResponse, routing::get};
use serde::Deserialize;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let port = 8081;
    let app = Router::new().route("/", get(proxy_url));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Deserialize)]
struct BandwidthHeroParams {
    url: Option<String>,
    jpeg: Option<bool>,
    bw: Option<u32>,
    l: Option<u8>,
}

async fn proxy_url(
    Query(params): Query<BandwidthHeroParams>,
) -> Result<impl IntoResponse, axum::http::StatusCode> {
    let url = params
        .url
        .map(|url| Url::parse(url.as_str()).ok())
        .flatten();

    let Some(url) = url else {
        return Ok("bandwidth-hero-proxy".into_response());
    };
    let quality = params.l.unwrap_or(40);
    let bw = params.bw.map(|bw| bw != 0).unwrap_or(true);
    let webp = params.jpeg.map(|jpeg| !jpeg).unwrap_or(true);

    Ok(url.to_string().into_response())
}
