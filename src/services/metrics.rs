use std::time::Instant;

pub fn init_metrics() {
    metrics::describe_counter!("auth_login_total", "Total of successful logins");
    metrics::describe_counter!("auth_login_failed_total", "Total of failed logins");
    metrics::describe_counter!("auth_2fa_attempts_total", "Total of 2FA verification attempts");
    metrics::describe_histogram!("auth_login_latency_ms", "Login request latency in milliseconds");
    metrics::describe_histogram!("auth_refresh_latency_ms", "Refresh token latency in milliseconds");
}

pub fn increment_login_success() {
    let counter = metrics::counter!("auth_login_total");
    counter.increment(1);
}

pub fn increment_login_failed() {
    let counter = metrics::counter!("auth_login_failed_total");
    counter.increment(1);
}

pub fn increment_2fa_attempts() {
    let counter = metrics::counter!("auth_2fa_attempts_total");
    counter.increment(1);
}

pub fn record_login_latency(duration_ms: f64) {
    let histogram = metrics::histogram!("auth_login_latency_ms");
    histogram.record(duration_ms);
}

pub fn record_refresh_latency(duration_ms: f64) {
    let histogram = metrics::histogram!("auth_refresh_latency_ms");
    histogram.record(duration_ms);
}

pub struct LatencyTimer {
    start: Instant,
}

impl LatencyTimer {
    pub fn new() -> Self {
        Self { start: Instant::now() }
    }

    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }
}

impl Default for LatencyTimer {
    fn default() -> Self {
        Self::new()
    }
}