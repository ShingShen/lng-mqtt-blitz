use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

#[derive(Default)]
pub struct LngMetrics {
    pub sent: AtomicU64,
    pub recv: AtomicU64,
    pub errors: AtomicU64,
    pub active_conns: AtomicU64,
}

#[derive(Debug, Clone)]
pub struct ThroughputStats {
    pub sent_total: u64,
    pub recv_total: u64,
    pub errors_total: u64,
    pub active_conns: u64,
    pub msg_per_sec: f64,
    // pub error_per_sec: f64,
}

pub struct StatsCalculator {
    metrics: std::sync::Arc<LngMetrics>,
    last_sent: u64,
    last_errors: u64,
    last_time: Instant,
}

impl StatsCalculator {
    pub fn new(metrics: std::sync::Arc<LngMetrics>) -> Self {
        Self {
            metrics,
            last_sent: 0,
            last_errors: 0,
            last_time: Instant::now(),
        }
    }

    pub fn calculate(&mut self) -> ThroughputStats {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_time).as_secs_f64();
        
        let current_sent = self.metrics.sent.load(Ordering::Relaxed);
        let current_recv = self.metrics.recv.load(Ordering::Relaxed);
        let current_errors = self.metrics.errors.load(Ordering::Relaxed);
        let current_active = self.metrics.active_conns.load(Ordering::Relaxed);

        let sent_diff = current_sent.saturating_sub(self.last_sent);
        let _error_diff = current_errors.saturating_sub(self.last_errors);

        let stats = ThroughputStats {
            sent_total: current_sent,
            recv_total: current_recv,
            errors_total: current_errors,
            active_conns: current_active,
            msg_per_sec: if elapsed > 0.0 { sent_diff as f64 / elapsed } else { 0.0 },
            // error_per_sec: if elapsed > 0.0 { error_diff as f64 / elapsed } else { 0.0 },
        };

        self.last_sent = current_sent;
        self.last_errors = current_errors;
        self.last_time = now;

        stats
    }
}
