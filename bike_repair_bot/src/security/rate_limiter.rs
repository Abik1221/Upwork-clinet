use anyhow::Result;
use dashmap::DashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::models::RateLimitInfo;

/// Rate limiter for controlling request frequency
pub struct RateLimiter {
    /// Per-IP request tracking
    ip_requests: Arc<DashMap<IpAddr, RequestTracker>>,
    
    /// Configuration
    max_per_minute: u32,
    max_per_hour: u32,
}

/// Track requests for a single IP/user
#[derive(Debug, Clone)]
struct RequestTracker {
    /// Requests in the last minute
    minute_requests: Vec<Instant>,
    
    /// Requests in the last hour
    hour_requests: Vec<Instant>,
    
    /// Last cleanup time
    last_cleanup: Instant,
}

impl RequestTracker {
    fn new() -> Self {
        Self {
            minute_requests: Vec::new(),
            hour_requests: Vec::new(),
            last_cleanup: Instant::now(),
        }
    }

    /// Remove expired entries
    fn cleanup(&mut self) {
        let now = Instant::now();
        let one_minute_ago = now - Duration::from_secs(60);
        let one_hour_ago = now - Duration::from_secs(3600);

        self.minute_requests.retain(|&t| t > one_minute_ago);
        self.hour_requests.retain(|&t| t > one_hour_ago);
        self.last_cleanup = now;
    }

    /// Add a new request
    fn add_request(&mut self) {
        let now = Instant::now();
        
        // Cleanup if it's been more than 10 seconds
        if now.duration_since(self.last_cleanup).as_secs() > 10 {
            self.cleanup();
        }

        self.minute_requests.push(now);
        self.hour_requests.push(now);
    }

    /// Check if request would exceed limits
    fn check_limits(&mut self, max_per_minute: u32, max_per_hour: u32) -> bool {
        self.cleanup();
        
        let minute_count = self.minute_requests.len() as u32;
        let hour_count = self.hour_requests.len() as u32;

        minute_count < max_per_minute && hour_count < max_per_hour
    }

    /// Get rate limit info
    fn get_info(&mut self, max_per_minute: u32, max_per_hour: u32) -> RateLimitInfo {
        self.cleanup();
        
        let minute_count = self.minute_requests.len() as u32;
        let hour_count = self.hour_requests.len() as u32;

        // Calculate reset time
        let reset_in_seconds = if minute_count >= max_per_minute {
            self.minute_requests
                .first()
                .map(|&t| {
                    let elapsed = Instant::now().duration_since(t).as_secs();
                    60u64.saturating_sub(elapsed)
                })
                .unwrap_or(60)
        } else if hour_count >= max_per_hour {
            self.hour_requests
                .first()
                .map(|&t| {
                    let elapsed = Instant::now().duration_since(t).as_secs();
                    3600u64.saturating_sub(elapsed)
                })
                .unwrap_or(3600)
        } else {
            0
        };

        RateLimitInfo {
            remaining_minute: max_per_minute.saturating_sub(minute_count),
            remaining_hour: max_per_hour.saturating_sub(hour_count),
            reset_in_seconds,
        }
    }
}

impl RateLimiter {
    pub fn new(max_per_minute: u32, max_per_hour: u32) -> Self {
        Self {
            ip_requests: Arc::new(DashMap::new()),
            max_per_minute,
            max_per_hour,
        }
    }

    /// Check if request is allowed and record it
    pub fn check_and_record(&self, ip: IpAddr) -> Result<RateLimitInfo> {
        let mut tracker = self.ip_requests
            .entry(ip)
            .or_insert_with(RequestTracker::new)
            .clone();

        // Check limits before adding
        if !tracker.check_limits(self.max_per_minute, self.max_per_hour) {
            let info = tracker.get_info(self.max_per_minute, self.max_per_hour);
            anyhow::bail!(
                "Rate limit exceeded. Try again in {} seconds",
                info.reset_in_seconds
            );
        }

        // Add the request
        tracker.add_request();
        
        // Update the stored tracker
        self.ip_requests.insert(ip, tracker.clone());

        // Return current limit info
        Ok(tracker.get_info(self.max_per_minute, self.max_per_hour))
    }

    /// Get current rate limit status without recording
    pub fn get_status(&self, ip: IpAddr) -> RateLimitInfo {
        self.ip_requests
            .get(&ip)
            .map(|e| {
                let mut tracker = e.clone();
                tracker.get_info(self.max_per_minute, self.max_per_hour)
            })
            .unwrap_or(RateLimitInfo {
                remaining_minute: self.max_per_minute,
                remaining_hour: self.max_per_hour,
                reset_in_seconds: 0,
            })
    }

    /// Cleanup old entries (should be called periodically)
    pub fn cleanup_old_entries(&self) {
        let now = Instant::now();
        let one_hour_ago = now - Duration::from_secs(3600);

        self.ip_requests.retain(|_, tracker| {
            // Remove entries that haven't been used in over an hour
            !tracker.hour_requests.is_empty() 
                && tracker.hour_requests.iter().any(|&t| t > one_hour_ago)
        });

        log::debug!("Rate limiter cleanup: {} active IPs", self.ip_requests.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_rate_limiter_allows_requests() {
        let limiter = RateLimiter::new(5, 10);
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // First 5 requests should succeed
        for _ in 0..5 {
            assert!(limiter.check_and_record(ip).is_ok());
        }

        // 6th request should fail (exceeds per-minute limit)
        assert!(limiter.check_and_record(ip).is_err());
    }
}
