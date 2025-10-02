use axum::{Router, routing::get};
use error::AppError;
use reqwest::{Client, redirect::Policy};
use std::{sync::Arc, time::Duration};

mod compress;
mod error;
mod headers;
mod params;
mod proxy;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenv::dotenv().ok();

    let app = app()?;

    let port = 8081;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

pub fn app() -> Result<Router, AppError> {
    init_tracing()?;
    let state = init_state()?;

    let app = Router::new()
        .route("/", get(proxy::proxy_url))
        .with_state(Arc::new(state));

    Ok(app)
}

fn init_tracing() -> Result<(), AppError> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

fn init_state() -> Result<Client, AppError> {
    let http_client = Client::builder()
        .timeout(Duration::from_millis(10000))
        .redirect(Policy::limited(5))
        .danger_accept_invalid_certs(true)
        .gzip(true)
        .cookie_store(true)
        .build()?;
    Ok(http_client)
}
