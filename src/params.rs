use crate::error::AppError;
use axum::{
    extract::{FromRequestParts, Query},
    http::request::Parts,
};
use serde::Deserialize;
use url::Url;

#[derive(Deserialize)]
pub struct RawBandwidthHeroParams {
    url: Option<String>,
    jpeg: Option<bool>,
    bw: Option<u32>,
    l: Option<u8>,
}

pub struct BandwidthHeroParams {
    pub url: Option<Url>,
    pub webp: bool,
    pub monochrome: bool,
    pub quality: u8,
}

impl<S> FromRequestParts<S> for BandwidthHeroParams
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(raw_params) =
            Query::<RawBandwidthHeroParams>::from_request_parts(parts, state).await?;

        let url = raw_params.url.and_then(|u| Url::parse(&u).ok());
        let webp = raw_params.jpeg.map(|j| !j).unwrap_or(true);
        let monochrome = raw_params.bw.map(|bw| bw != 0).unwrap_or(true);
        let quality = raw_params.l.unwrap_or(40);
        Ok(BandwidthHeroParams {
            url,
            webp,
            monochrome,
            quality,
        })
    }
}
