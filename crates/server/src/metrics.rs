//! Prometheus metrics for observability.
//!
//! Exposes counters, histograms, and gauges for monitoring:
//! - WebSocket connections
//! - Room lifecycle
//! - AI generation latency
//! - HTTP request duration
//! - Active games by mode

use prometheus::{
    register_counter_vec, register_gauge, register_histogram_vec, CounterVec, Gauge, HistogramVec,
};
use std::sync::OnceLock;

/// Global metrics registry.
static METRICS: OnceLock<Metrics> = OnceLock::new();

/// All application metrics.
pub struct Metrics {
    /// Active WebSocket connections by node.
    pub ws_connections_active: Gauge,
    /// Total games started by mode.
    pub games_started_total: CounterVec,
    /// Question generation latency by provider and status.
    pub ai_generation_seconds: HistogramVec,
    /// HTTP request duration by method, path, and status.
    pub http_request_duration_seconds: HistogramVec,
    /// Active rooms by mode.
    pub room_active_total: Gauge,
    /// Total answers submitted.
    pub answers_total: CounterVec,
}

impl Metrics {
    /// Initialize metrics (idempotent).
    pub fn global() -> &'static Metrics {
        METRICS.get_or_init(|| {
            let ws_connections_active = register_gauge!(
                "brainstorm_ws_connections_active",
                "Active WebSocket connections"
            )
            .expect("failed to register ws_connections_active");

            let games_started_total = register_counter_vec!(
                "brainstorm_games_started_total",
                "Total games started",
                &["mode"]
            )
            .expect("failed to register games_started_total");

            let ai_generation_seconds = register_histogram_vec!(
                "brainstorm_ai_generation_seconds",
                "AI question generation latency",
                &["provider", "status"],
                vec![0.1, 0.25, 0.5, 1.0, 2.0, 3.0, 5.0, 10.0]
            )
            .expect("failed to register ai_generation_seconds");

            let http_request_duration_seconds = register_histogram_vec!(
                "brainstorm_http_request_duration_seconds",
                "HTTP request duration",
                &["method", "path", "status"],
                vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
            )
            .expect("failed to register http_request_duration_seconds");

            let room_active_total = register_gauge!("brainstorm_room_active_total", "Active rooms")
                .expect("failed to register room_active_total");

            let answers_total = register_counter_vec!(
                "brainstorm_answers_total",
                "Total answers submitted",
                &["correct"]
            )
            .expect("failed to register answers_total");

            Metrics {
                ws_connections_active,
                games_started_total,
                ai_generation_seconds,
                http_request_duration_seconds,
                room_active_total,
                answers_total,
            }
        })
    }

    /// Record a game start.
    pub fn record_game_start(&self, mode: &str) {
        self.games_started_total.with_label_values(&[mode]).inc();
    }

    /// Record AI generation latency.
    pub fn record_ai_generation(&self, provider: &str, status: &str, duration_secs: f64) {
        self.ai_generation_seconds
            .with_label_values(&[provider, status])
            .observe(duration_secs);
    }

    /// Record an answer.
    pub fn record_answer(&self, correct: bool) {
        let label = if correct { "true" } else { "false" };
        self.answers_total.with_label_values(&[label]).inc();
    }

    /// Update active room count.
    pub fn set_active_rooms(&self, count: usize) {
        self.room_active_total.set(count as f64);
    }
}

/// Record one HTTP request in the duration histogram.
pub fn record_http_request(method: &str, route: &str, status: u16, duration_secs: f64) {
    Metrics::global()
        .http_request_duration_seconds
        .with_label_values(&[method, route, &status.to_string()])
        .observe(duration_secs);
}

/// Encode all metrics in Prometheus text format.
pub fn encode_metrics() -> String {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let mut buffer = Vec::new();
    if encoder.encode(&prometheus::gather(), &mut buffer).is_err() {
        return String::new();
    }
    String::from_utf8(buffer).unwrap_or_default()
}
