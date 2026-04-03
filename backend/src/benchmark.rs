//! Benchmark utilities for measuring performance.
//! 
//! Provides metrics collection for:
//! - Packets per second throughput
//! - Classification latency
//! - Memory usage
//! - WebSocket message throughput
//!
//! Usage:
//! ```rust
//! use backend::benchmark::{BenchmarkMetrics, BenchmarkRecorder};
//!
//! let metrics = BenchmarkMetrics::default();
//! metrics.record_classification(latency_ns);
//! metrics.record_packet_processed();
//! println!("PPS: {}", metrics.packets_per_second());
//! ```

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Metrics for tracking performance.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchmarkMetrics {
    /// Total packets processed
    pub total_packets: u64,
    
    /// Total classification operations
    pub total_classifications: u64,
    
    /// Total bytes processed
    pub total_bytes: u64,
    
    /// Sum of all classification latencies (nanoseconds)
    pub total_latency_ns: u64,
    
    /// Number of latency samples
    pub latency_samples: u64,
    
    /// WebSocket messages sent
    pub ws_messages_sent: u64,
    
    /// WebSocket messages received
    pub ws_messages_received: u64,
    
    /// Start time for calculating rates
    pub start_time_ns: u64,
    
    /// Peak memory usage (bytes) - tracked externally
    #[serde(default)]
    pub peak_memory_bytes: usize,
}

impl BenchmarkMetrics {
    /// Create new metrics with current timestamp.
    pub fn new() -> Self {
        Self {
            start_time_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_nanos() as u64,
            ..Default::default()
        }
    }
    
    /// Record a packet processed.
    pub fn record_packet(&mut self, packet_size: u32) {
        self.total_packets += 1;
        self.total_bytes += packet_size as u64;
    }
    
    /// Record a classification with latency.
    pub fn record_classification(&mut self, latency_ns: u64) {
        self.total_classifications += 1;
        self.total_latency_ns += latency_ns;
        self.latency_samples += 1;
    }
    
    /// Record a WebSocket message sent.
    pub fn record_ws_sent(&mut self) {
        self.ws_messages_sent += 1;
    }
    
    /// Record a WebSocket message received.
    pub fn record_ws_received(&mut self) {
        self.ws_messages_received += 1;
    }
    
    /// Calculate packets per second.
    pub fn packets_per_second(&self) -> f64 {
        let elapsed = self.elapsed_seconds();
        if elapsed > 0.0 {
            self.total_packets as f64 / elapsed
        } else {
            0.0
        }
    }
    
    /// Calculate bytes per second.
    pub fn bytes_per_second(&self) -> f64 {
        let elapsed = self.elapsed_seconds();
        if elapsed > 0.0 {
            self.total_bytes as f64 / elapsed
        } else {
            0.0
        }
    }
    
    /// Calculate average classification latency in microseconds.
    pub fn avg_latency_us(&self) -> f64 {
        if self.latency_samples > 0 {
            (self.total_latency_ns / self.latency_samples) as f64 / 1000.0
        } else {
            0.0
        }
    }
    
    /// Calculate median latency (approximation using simple approach).
    // Note: This would need a proper histogram for accurate median
    pub fn median_latency_us(&self) -> f64 {
        // Simplified - for accurate median, we'd track percentiles
        self.avg_latency_us()
    }
    
    /// Calculate WebSocket messages per second.
    pub fn ws_messages_per_second(&self) -> f64 {
        let elapsed = self.elapsed_seconds();
        if elapsed > 0.0 {
            self.ws_messages_sent as f64 / elapsed
        } else {
            0.0
        }
    }
    
    fn elapsed_seconds(&self) -> f64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_nanos() as u64;
        
        if now > self.start_time_ns {
            (now - self.start_time_ns) as f64 / 1_000_000_000.0
        } else {
            0.0
        }
    }
    
    /// Get summary as formatted string.
    pub fn summary(&self) -> String {
        format!(
            "Packets: {} | PPS: {:.0} | Bytes/s: {:.0} | Avg Latency: {:.2}μs | WS Msg/s: {:.0}",
            self.total_packets,
            self.packets_per_second(),
            self.bytes_per_second(),
            self.avg_latency_us(),
            self.ws_messages_per_second()
        )
    }
}

/// Thread-safe benchmark recorder.
/// 
/// Uses atomic operations for recording metrics from multiple threads.
#[derive(Debug)]
pub struct BenchmarkRecorder {
    total_packets: AtomicU64,
    total_bytes: AtomicU64,
    total_latency_ns: AtomicU64,
    latency_samples: AtomicU64,
    ws_sent: AtomicU64,
    ws_received: AtomicU64,
    start_time: Instant,
}

impl Default for BenchmarkRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl BenchmarkRecorder {
    /// Create new recorder.
    pub fn new() -> Self {
        Self {
            total_packets: AtomicU64::new(0),
            total_bytes: AtomicU64::new(0),
            total_latency_ns: AtomicU64::new(0),
            latency_samples: AtomicU64::new(0),
            ws_sent: AtomicU64::new(0),
            ws_received: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }
    
    /// Record a packet.
    #[inline]
    pub fn record_packet(&self, size: u32) {
        self.total_packets.fetch_add(1, Ordering::Relaxed);
        self.total_bytes.fetch_add(size as u64, Ordering::Relaxed);
    }
    
    /// Record classification latency.
    #[inline]
    pub fn record_latency(&self, latency_ns: u64) {
        self.total_latency_ns.fetch_add(latency_ns, Ordering::Relaxed);
        self.latency_samples.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record WebSocket message sent.
    #[inline]
    pub fn record_ws_sent(&self) {
        self.ws_sent.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record WebSocket message received.
    #[inline]
    pub fn record_ws_received(&self) {
        self.ws_received.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get current metrics snapshot.
    pub fn snapshot(&self) -> BenchmarkMetrics {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        
        let packets = self.total_packets.load(Ordering::Relaxed);
        let bytes = self.total_bytes.load(Ordering::Relaxed);
        let latency = self.total_latency_ns.load(Ordering::Relaxed);
        let samples = self.latency_samples.load(Ordering::Relaxed);
        
        BenchmarkMetrics {
            total_packets: packets,
            total_classifications: samples,
            total_bytes: bytes,
            total_latency_ns: latency,
            latency_samples: samples,
            ws_messages_sent: self.ws_sent.load(Ordering::Relaxed),
            ws_messages_received: self.ws_received.load(Ordering::Relaxed),
            start_time_ns: 0, // Not tracked in atomic version
            peak_memory_bytes: 0,
        }
    }
    
    /// Print current stats.
    pub fn print_stats(&self) {
        let metrics = self.snapshot();
        println!("[Benchmark] {}", metrics.summary());
    }
}

/// Simple benchmark runner for testing.
pub struct BenchmarkRunner {
    pub name: String,
    pub iterations: u64,
    pub warmup_iterations: u64,
}

impl BenchmarkRunner {
    /// Run a benchmark function and return timing.
    pub fn run<F, R>(&self, mut f: F) -> (R, Duration) 
    where
        F: FnMut() -> R,
    {
        // Warmup
        for _ in 0..self.warmup_iterations {
            let _ = f();
        }
        
        // Actual benchmark
        let start = Instant::now();
        for _ in 0..self.iterations {
            let _ = f();
        }
        let duration = start.elapsed();
        
        let result = f(); // Final call for return value
        
        (result, duration)
    }
    
    /// Calculate operations per second.
    pub fn ops_per_second(iterations: u64, duration: Duration) -> f64 {
        let secs = duration.as_secs_f64();
        if secs > 0.0 {
            iterations as f64 / secs
        } else {
            0.0
        }
    }
    
    /// Calculate nanoseconds per operation.
    pub fn ns_per_op(iterations: u64, duration: Duration) -> f64 {
        let total_ns = duration.as_nanos() as f64;
        if iterations > 0 {
            total_ns / iterations as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics_calculation() {
        let mut metrics = BenchmarkMetrics::new();
        
        metrics.record_packet(512);
        metrics.record_packet(1024);
        metrics.record_classification(1000); // 1μs
        metrics.record_classification(2000); // 2μs
        
        assert_eq!(metrics.total_packets, 2);
        assert_eq!(metrics.total_bytes, 1536);
        assert_eq!(metrics.latency_samples, 2);
        
        // Average should be 1.5μs
        let avg = metrics.avg_latency_us();
        assert!((avg - 1.5).abs() < 0.01);
    }
    
    #[test]
    fn test_recorder_atomic() {
        let recorder = BenchmarkRecorder::new();
        
        recorder.record_packet(512);
        recorder.record_packet(256);
        recorder.record_latency(1000);
        
        let metrics = recorder.snapshot();
        assert_eq!(metrics.total_packets, 2);
    }
}