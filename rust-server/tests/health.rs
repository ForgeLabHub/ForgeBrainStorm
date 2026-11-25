//! Integration test: health and readiness endpoints.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

// Note: These tests require a running Postgres instance.
// Run with: cargo test --test health -- --ignored
// Or set up a test database via docker compose.

#[tokio::test]
#[ignore = "requires database"]
async fn test_health_endpoint() {
    // TODO: build test app with mock state
    // let app = build_test_app().await;
    // let response = app
    //     .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
    //     .await
    //     .unwrap();
    // assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_ready_endpoint() {
    // TODO: verify /readyz returns 200 when DB is healthy
}
