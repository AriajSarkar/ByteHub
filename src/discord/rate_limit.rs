//! Per-guild rate limiting for expensive Discord commands
//!
//! Prevents command spam that could cause database write conflicts
//! and excessive resource usage.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Simple per-guild rate limiter using a fixed window approach
pub struct RateLimiter {
    /// Guild ID -> (window_start, request_count)
    state: Mutex<HashMap<String, (Instant, u32)>>,
    window: Duration,
    max_requests: u32,
}

impl RateLimiter {
    /// Create a new rate limiter
    ///
    /// # Arguments
    /// * `window_secs` - Time window in seconds
    /// * `max_requests` - Maximum requests allowed per window
    pub fn new(window_secs: u64, max_requests: u32) -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
            window: Duration::from_secs(window_secs),
            max_requests,
        }
    }

    /// Check if a request is allowed for the given guild
    ///
    /// # Returns
    /// * `Ok(())` - Request is allowed
    /// * `Err(seconds)` - Rate limited, wait this many seconds
    pub fn check(&self, guild_id: &str) -> Result<(), u64> {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();

        if let Some((window_start, count)) = state.get_mut(guild_id) {
            if now.duration_since(*window_start) > self.window {
                // Window expired, reset
                *window_start = now;
                *count = 1;
                Ok(())
            } else if *count >= self.max_requests {
                // Rate limited - calculate wait time
                let elapsed = now.duration_since(*window_start).as_secs();
                let wait = self.window.as_secs().saturating_sub(elapsed);
                Err(wait.max(1)) // At least 1 second
            } else {
                // Increment count
                *count += 1;
                Ok(())
            }
        } else {
            // First request for this guild
            state.insert(guild_id.to_string(), (now, 1));
            Ok(())
        }
    }

    /// Clean up expired entries (call periodically to prevent memory bloat)
    #[allow(dead_code)]
    pub fn cleanup(&self) {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();
        state.retain(|_, (window_start, _)| now.duration_since(*window_start) <= self.window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new(60, 5);

        for _ in 0..5 {
            assert!(limiter.check("guild1").is_ok());
        }
    }

    #[test]
    fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new(60, 2);

        assert!(limiter.check("guild1").is_ok());
        assert!(limiter.check("guild1").is_ok());
        assert!(limiter.check("guild1").is_err());
    }

    #[test]
    fn test_rate_limiter_separate_guilds() {
        let limiter = RateLimiter::new(60, 1);

        assert!(limiter.check("guild1").is_ok());
        assert!(limiter.check("guild2").is_ok());
        assert!(limiter.check("guild1").is_err());
        assert!(limiter.check("guild2").is_err());
    }
}
