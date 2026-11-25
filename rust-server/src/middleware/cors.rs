//! CORS layer configured from [`Settings::cors_origins`].
//!
//! Refuses to start with `*` in production — the check lives in
//! [`Settings::validate`], so by the time we reach this code the origins
//! are already guaranteed to be explicit.

use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::config::Settings;

pub fn build_cors_layer(settings: &Settings) -> CorsLayer {
    let origins: Vec<axum::http::HeaderValue> = settings
        .cors_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    let allow = if origins.is_empty() {
        AllowOrigin::default()
    } else {
        AllowOrigin::list(origins)
    };

    CorsLayer::new()
        .allow_origin(allow)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
        .allow_credentials(true)
        .expose_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::SET_COOKIE,
        ])
}
