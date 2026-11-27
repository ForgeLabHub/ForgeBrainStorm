//! Records every HTTP request in the Prometheus duration histogram.
//!
//! The label is the matched route pattern (`/api/rooms/:code`) rather than the
//! raw path, so room codes and usernames do not explode label cardinality.

use std::time::Instant;

use axum::extract::{MatchedPath, Request};
use axum::middleware::Next;
use axum::response::Response;

pub async fn track_http_request(request: Request, next: Next) -> Response {
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(|path| path.as_str().to_string())
        .unwrap_or_else(|| "unmatched".to_string());
    let method = request.method().to_string();
    let started = Instant::now();

    let response = next.run(request).await;

    crate::metrics::record_http_request(
        &method,
        &route,
        response.status().as_u16(),
        started.elapsed().as_secs_f64(),
    );
    response
}
