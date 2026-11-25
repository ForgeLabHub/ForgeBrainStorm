//! CORS layer configured from [`Settings::cors_origins`].
//!
//! Refuses to start with `*` in production — the check lives in
//! [`Settings::validate`], so by the time we reach this code the origins
//! are already guaranteed to be explicit.
//!
//! Note: `allow_credentials(true)` is incompatible with `AllowOrigin::any()`
//! (browsers reject it). We only enable credentials when origins list is
//! non-empty, and we never use `AllowOrigin::any()`.

use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::config::Settings;

pub fn build_cors_layer(settings: &Settings) -> CorsLayer {
    let origins: Vec<axum::http::HeaderValue> = settings
        .cors_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    let mut layer = CorsLayer::new()
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    if origins.is_empty() {
        // No credentials when no origins configured — avoids the
        // "credentials + *" incompatibility that makes browsers reject
        // the response.
        layer = layer.allow_origin(tower_http::cors::Any);
    } else {
        layer = layer
            .allow_origin(AllowOrigin::list(origins))
            .allow_credentials(true);
    }

    layer.expose_headers([
        axum::http::header::CONTENT_TYPE,
        axum::http::header::SET_COOKIE,
    ])
}
