use axum::http::{
    HeaderMap as AxumHeaderMap, HeaderName as AxumHeaderName, HeaderValue as AxumHeaderValue,
};
use reqwest::header::{
    HeaderMap as ReqwestHeaderMap, HeaderName as ReqwestHeaderName,
    HeaderValue as ReqwestHeaderValue,
};

// Utility functions for header conversion
fn axum_to_reqwest_headers(
    axum_headers: &AxumHeaderMap,
) -> Result<ReqwestHeaderMap, Box<dyn std::error::Error>> {
    let mut reqwest_headers = ReqwestHeaderMap::new();

    for (name, value) in axum_headers.iter() {
        let reqwest_name = ReqwestHeaderName::from_bytes(name.as_str().as_bytes())?;
        let reqwest_value = ReqwestHeaderValue::from_bytes(value.as_bytes())?;
        reqwest_headers.insert(reqwest_name, reqwest_value);
    }

    Ok(reqwest_headers)
}

pub fn reqwest_to_axum_headers(
    reqwest_headers: &ReqwestHeaderMap,
) -> Result<AxumHeaderMap, anyhow::Error> {
    let mut axum_headers = AxumHeaderMap::new();

    for (name, value) in reqwest_headers.iter() {
        let axum_name = AxumHeaderName::from_bytes(name.as_str().as_bytes())?;
        let axum_value = AxumHeaderValue::from_bytes(value.as_bytes())?;
        axum_headers.insert(axum_name, axum_value);
    }

    Ok(axum_headers)
}

// Selective header forwarding utility
pub fn forward_specific_headers(
    source: &AxumHeaderMap,
    target: &mut ReqwestHeaderMap,
    headers_to_forward: &[&str],
) -> Result<(), anyhow::Error> {
    for header_name in headers_to_forward {
        if let Some(value) = source.get(*header_name) {
            let reqwest_name = ReqwestHeaderName::from_bytes(header_name.as_bytes())?;
            let reqwest_value = ReqwestHeaderValue::from_bytes(value.as_bytes())?;
            target.insert(reqwest_name, reqwest_value);
        }
    }
    Ok(())
}
