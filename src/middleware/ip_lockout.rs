use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;

const MAX_FAILED_ATTEMPTS: u32 = 5;
const LOCKOUT_DURATION: Duration = Duration::from_secs(15 * 60);

#[derive(Clone)]
pub struct IpLockout {
    inner: Arc<Inner>,
}

struct Inner {
    failed_attempts: DashMap<String, FailedAttempt>,
    locked_ips: DashMap<String, Instant>,
}

#[derive(Clone, Copy)]
struct FailedAttempt {
    count: u32,
    first_attempt: Instant,
}

impl IpLockout {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                failed_attempts: DashMap::new(),
                locked_ips: DashMap::new(),
            }),
        }
    }

    pub fn is_locked(&self, ip: &str) -> bool {
        if let Some(locked_until) = self.inner.locked_ips.get(ip) {
            if Instant::now() < *locked_until {
                return true;
            } else {
                self.inner.locked_ips.remove(ip);
                self.inner.failed_attempts.remove(ip);
            }
        }
        false
    }

    pub fn get_remaining_lockout_secs(&self, ip: &str) -> Option<u64> {
        self.inner.locked_ips.get(ip).map(|guard| {
            let locked_until: &Instant = &*guard;
            let remaining = locked_until.duration_since(Instant::now());
            remaining.as_secs()
        })
    }

    pub fn record_failure(&self, ip: &str) {
        let now = Instant::now();

        let should_lock = {
            let mut entry = self.inner.failed_attempts.entry(ip.to_string()).or_insert_with(|| FailedAttempt {
                count: 0,
                first_attempt: now,
            });

            entry.count += 1;

            if entry.count >= MAX_FAILED_ATTEMPTS {
                true
            } else {
                false
            }
        };

        if should_lock {
            self.inner.locked_ips.insert(ip.to_string(), now + LOCKOUT_DURATION);
        }
    }

    pub fn record_success(&self, ip: &str) {
        self.inner.failed_attempts.remove(ip);
    }

    pub fn clear_lockout(&self, ip: &str) {
        self.inner.failed_attempts.remove(ip);
        self.inner.locked_ips.remove(ip);
    }
}

impl Default for IpLockout {
    fn default() -> Self {
        Self::new()
    }
}
