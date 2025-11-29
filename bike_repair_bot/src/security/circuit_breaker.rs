use anyhow::Result;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    /// Normal operation - requests pass through
    Closed,
    
    /// Too many failures - requests blocked
    Open,
    
    /// Testing if service recovered - limited requests allowed
    HalfOpen,
}

/// Circuit breaker to protect against cascading failures
pub struct CircuitBreaker {
    /// Current state
    state: Arc<RwLock<CircuitState>>,
    
    /// Consecutive failure count
    failure_count: Arc<AtomicU32>,
    
    /// Failure threshold before opening circuit
    threshold: u32,
    
    /// Time when circuit was opened
    opened_at: Arc<RwLock<Option<Instant>>>,
    
    /// Timeout before attempting recovery
    timeout: Duration,
    
    /// Total requests
    total_requests: Arc<AtomicU64>,
    
    /// Total failures
    total_failures: Arc<AtomicU64>,
}

impl CircuitBreaker {
    pub fn new(threshold: u32, timeout_seconds: u64) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(AtomicU32::new(0)),
            threshold,
            opened_at: Arc::new(RwLock::new(None)),
            timeout: Duration::from_secs(timeout_seconds),
            total_requests: Arc::new(AtomicU64::new(0)),
            total_failures: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Check if request should be allowed
    pub async fn check_request(&self) -> Result<()> {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        
        let state = *self.state.read().await;

        match state {
            CircuitState::Closed => {
                // Normal operation - allow request
                Ok(())
            }
            CircuitState::Open => {
                // Check if timeout has elapsed
                let opened_at = self.opened_at.read().await;
                if let Some(opened_time) = *opened_at {
                    if Instant::now().duration_since(opened_time) >= self.timeout {
                        // Transition to half-open
                        drop(opened_at);
                        *self.state.write().await = CircuitState::HalfOpen;
                        log::info!("Circuit breaker transitioning to half-open state");
                        Ok(())
                    } else {
                        anyhow::bail!(
                            "Service temporarily unavailable (circuit breaker open). \
                            Please try again in a moment."
                        )
                    }
                } else {
                    anyhow::bail!("Service temporarily unavailable")
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited requests to test service
                Ok(())
            }
        }
    }

    /// Record a successful request
    pub async fn record_success(&self) {
        let state = *self.state.read().await;

        match state {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::Relaxed);
            }
            CircuitState::HalfOpen => {
                // Success in half-open state - close the circuit
                log::info!("Circuit breaker closing after successful request");
                *self.state.write().await = CircuitState::Closed;
                self.failure_count.store(0, Ordering::Relaxed);
                *self.opened_at.write().await = None;
            }
            CircuitState::Open => {
                // Shouldn't happen, but reset anyway
                self.failure_count.store(0, Ordering::Relaxed);
            }
        }
    }

    /// Record a failed request
    pub async fn record_failure(&self) {
        self.total_failures.fetch_add(1, Ordering::Relaxed);
        
        let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        let state = *self.state.read().await;

        match state {
            CircuitState::Closed => {
                if failures >= self.threshold {
                    // Too many failures - open the circuit
                    log::warn!(
                        "Circuit breaker opening after {} consecutive failures",
                        failures
                    );
                    drop(state);
                    *self.state.write().await = CircuitState::Open;
                    *self.opened_at.write().await = Some(Instant::now());
                }
            }
            CircuitState::HalfOpen => {
                // Failure in half-open - back to open
                log::warn!("Circuit breaker reopening after failure in half-open state");
                drop(state);
                *self.state.write().await = CircuitState::Open;
                *self.opened_at.write().await = Some(Instant::now());
            }
            CircuitState::Open => {
                // Already open, do nothing
            }
        }
    }

    /// Get current state
    pub async fn get_state(&self) -> CircuitState {
        *self.state.read().await
    }

    /// Get statistics
    pub async fn get_stats(&self) -> CircuitStats {
        CircuitStats {
            state: *self.state.read().await,
            failure_count: self.failure_count.load(Ordering::Relaxed),
            threshold: self.threshold,
            total_requests: self.total_requests.load(Ordering::Relaxed),
            total_failures: self.total_failures.load(Ordering::Relaxed),
        }
    }

    /// Manually reset the circuit breaker
    pub async fn reset(&self) {
        log::info!("Manually resetting circuit breaker");
        *self.state.write().await = CircuitState::Closed;
        self.failure_count.store(0, Ordering::Relaxed);
        *self.opened_at.write().await = None;
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone)]
pub struct CircuitStats {
    pub state: CircuitState,
    pub failure_count: u32,
    pub threshold: u32,
    pub total_requests: u64,
    pub total_failures: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_opens_on_failures() {
        let breaker = CircuitBreaker::new(3, 5);

        // Record failures
        for _ in 0..3 {
            breaker.record_failure().await;
        }

        // Circuit should be open now
        assert_eq!(breaker.get_state().await, CircuitState::Open);
        assert!(breaker.check_request().await.is_err());
    }

    #[tokio::test]
    async fn test_circuit_breaker_closes_on_success() {
        let breaker = CircuitBreaker::new(2, 1);

        // Open the circuit
        breaker.record_failure().await;
        breaker.record_failure().await;
        assert_eq!(breaker.get_state().await, CircuitState::Open);

        // Wait for timeout
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Try request (should transition to half-open)
        assert!(breaker.check_request().await.is_ok());
        assert_eq!(breaker.get_state().await, CircuitState::HalfOpen);

        // Success should close it
        breaker.record_success().await;
        assert_eq!(breaker.get_state().await, CircuitState::Closed);
    }
}
