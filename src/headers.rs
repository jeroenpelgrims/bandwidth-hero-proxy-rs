use std::collections::HashSet;

use axum::http::{HeaderMap, HeaderName, HeaderValue};

pub fn filter_headers(source: HeaderMap, keys: Vec<&str>) -> HeaderMap {
    let keys: HashSet<&str> = HashSet::from_iter(keys);
    let filtered: Vec<(HeaderName, HeaderValue)> = source
        .iter()
        .filter(|(key, _)| keys.contains(key.as_str()))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();

    HeaderMap::from_iter(filtered)
}
