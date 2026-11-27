//! CORS layer configured from [`Settings::cors_origins`].
//!
//! `Access-Control-Allow-Credentials: true` cannot be combined with wildcard
//! `Origin`, `Methods`, or `Headers` — tower-http panics on that combination.
//! We therefore always send explicit method/header lists, and use a wildcard
//! origin only when the caller opted in with `*` (which disables credentials).

use axum::http::{header, HeaderValue, Method};
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

use crate::config::Settings;

const ALLOWED_METHODS: [Method; 6] = [
    Method::GET,
    Method::POST,
    Method::PUT,
    Method::PATCH,
    Method::DELETE,
    Method::OPTIONS,
];

const ALLOWED_HEADERS: [header::HeaderName; 6] = [
    header::CONTENT_TYPE,
    header::AUTHORIZATION,
    header::ACCEPT,
    header::ORIGIN,
    header::COOKIE,
    header::SET_COOKIE,
];

pub fn build_cors_layer(settings: &Settings) -> CorsLayer {
    let base = CorsLayer::new()
        .allow_methods(ALLOWED_METHODS)
        .allow_headers(ALLOWED_HEADERS)
        .expose_headers([header::CONTENT_TYPE, header::SET_COOKIE]);

    if settings.cors_origins.iter().any(|o| o == "*") {
        base.allow_origin(Any)
    } else {
        let origins: Vec<HeaderValue> = settings
            .cors_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        base.allow_origin(AllowOrigin::list(origins))
            .allow_credentials(true)
    }
}
