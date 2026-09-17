use {
    reqwest::{header::HeaderMap, Method},
    serde::Deserialize,
    std::{collections::HashMap, time::Duration},
    tokio::{
        sync::Mutex,
        time::{sleep, Instant},
    },
};

const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(10);

/// Tracks Stoat REST rate-limit buckets and reserves capacity before requests are sent.
#[derive(Debug)]
pub struct RateLimiter {
    state: Mutex<RateLimitState>,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self {
            state: Mutex::new(RateLimitState::default()),
        }
    }
}

#[derive(Debug, Default)]
struct RateLimitState {
    buckets: HashMap<String, BucketState>,
}

#[derive(Debug, Clone)]
struct BucketState {
    limit: u32,
    remaining: u32,
    reset_at: Instant,
}

#[derive(Debug, Deserialize)]
pub struct RateLimitedResponse {
    /// Milliseconds until calls are replenished.
    pub retry_after: u64,
}

impl RateLimiter {
    /// Wait for and reserve one request slot for a route.
    ///
    /// This compatibility method now reserves capacity atomically. Prefer [`RateLimiter::acquire`]
    /// when the caller needs to observe how long it waited.
    pub async fn wait(&self, method: &Method, path: &str) {
        let _ = self.acquire(method, path).await;
    }

    /// Atomically reserve one request slot for a route, waiting for the current fixed window to
    /// reset when no capacity remains.
    ///
    /// The returned duration is the total amount of time spent waiting locally.
    pub async fn acquire(&self, method: &Method, path: &str) -> Duration {
        let route = route_key(method, path);
        let fallback = static_bucket(method, path);
        let mut total_wait = Duration::ZERO;

        loop {
            let wait_for = {
                let mut state = self.state.lock().await;
                let now = Instant::now();
                let bucket = state.buckets.entry(route.clone()).or_insert_with(|| {
                    let limit = fallback.limit;
                    BucketState {
                        limit,
                        remaining: limit,
                        reset_at: now + RATE_LIMIT_WINDOW,
                    }
                });

                if now >= bucket.reset_at {
                    bucket.remaining = bucket.limit;
                    bucket.reset_at = now + RATE_LIMIT_WINDOW;
                }

                if bucket.remaining > 0 {
                    bucket.remaining -= 1;
                    None
                } else {
                    Some(bucket.reset_at.saturating_duration_since(now))
                }
            };

            match wait_for {
                Some(delay) if !delay.is_zero() => {
                    total_wait += delay;
                    sleep(delay).await;
                }
                _ => return total_wait,
            }
        }
    }

    /// Update tracked bucket state from Stoat rate-limit headers.
    ///
    /// Local reservations are kept conservatively when concurrent responses arrive out of order,
    /// preventing an older response from increasing the amount of capacity Kaho believes remains.
    pub async fn update_from_headers(&self, method: &Method, path: &str, headers: &HeaderMap) {
        // The server bucket ID is useful protocol metadata, but Kaho deliberately keeps local
        // reservations under its canonical documented route key. Moving a live bucket to a new
        // key when the first concurrent response arrives would temporarily forget reservations
        // made by the other in-flight requests and could overshoot the limit.
        if header_str(headers, "X-RateLimit-Bucket").is_none() {
            return;
        }

        let header_limit = header_u32(headers, "X-RateLimit-Limit");
        let header_remaining = header_u32(headers, "X-RateLimit-Remaining");
        let header_reset_after = header_u64(headers, "X-RateLimit-Reset-After");
        let fallback = static_bucket(method, path);
        let route = route_key(method, path);
        let now = Instant::now();

        let mut state = self.state.lock().await;
        let previous = state.buckets.get(&route).cloned();
        let limit = header_limit
            .or_else(|| previous.as_ref().map(|bucket| bucket.limit))
            .unwrap_or(fallback.limit);
        let reset_at = header_reset_after
            .map(|milliseconds| now + Duration::from_millis(milliseconds))
            .or_else(|| previous.as_ref().map(|bucket| bucket.reset_at))
            .unwrap_or(now + RATE_LIMIT_WINDOW);
        let reported_remaining = header_remaining
            .or_else(|| previous.as_ref().map(|bucket| bucket.remaining))
            .unwrap_or(limit)
            .min(limit);

        let remaining = match previous {
            Some(previous) if previous.reset_at > now => previous.remaining.min(reported_remaining),
            _ => reported_remaining,
        };

        state.buckets.insert(
            route,
            BucketState {
                limit,
                remaining,
                reset_at,
            },
        );
    }

    /// Mark the route as rate-limited for the provided retry interval.
    pub async fn update_retry_after(&self, method: &Method, path: &str, retry_after_ms: u64) {
        let route = route_key(method, path);
        let fallback = static_bucket(method, path);

        let mut state = self.state.lock().await;
        let limit = state
            .buckets
            .get(&route)
            .map(|bucket| bucket.limit)
            .unwrap_or(fallback.limit);
        state.buckets.insert(
            route,
            BucketState {
                limit,
                remaining: 0,
                reset_at: Instant::now() + Duration::from_millis(retry_after_ms),
            },
        );
    }
}

#[derive(Debug, Clone, Copy)]
struct StaticBucket {
    name: &'static str,
    limit: u32,
}

fn static_bucket(method: &Method, path: &str) -> StaticBucket {
    let path = clean_url_path(path);

    if method == Method::PATCH && matches_pattern(&path, "/users/:id") {
        return StaticBucket {
            name: "PATCH /users/:id",
            limit: 2,
        };
    }

    if method == Method::POST && matches_pattern(&path, "/channels/:id/messages") {
        return StaticBucket {
            name: "POST /channels/:id/messages",
            limit: 10,
        };
    }

    if method == Method::DELETE && path.starts_with("/auth") {
        return StaticBucket {
            name: "DELETE /auth",
            limit: 255,
        };
    }

    if matches_pattern(&path, "/users/:id/default_avatar") {
        StaticBucket {
            name: "/users/:id/default_avatar",
            limit: 255,
        }
    } else if path.starts_with("/users") {
        StaticBucket {
            name: "/users",
            limit: 20,
        }
    } else if path.starts_with("/bots") {
        StaticBucket {
            name: "/bots",
            limit: 10,
        }
    } else if path.starts_with("/channels") {
        StaticBucket {
            name: "/channels",
            limit: 15,
        }
    } else if path.starts_with("/servers") {
        StaticBucket {
            name: "/servers",
            limit: 5,
        }
    } else if path.starts_with("/auth") {
        StaticBucket {
            name: "/auth",
            limit: 3,
        }
    } else if path.starts_with("/safety/report") {
        StaticBucket {
            name: "/safety/report",
            limit: 3,
        }
    } else if path.starts_with("/safety") {
        StaticBucket {
            name: "/safety",
            limit: 15,
        }
    } else if path.starts_with("/swagger") {
        StaticBucket {
            name: "/swagger",
            limit: 100,
        }
    } else {
        StaticBucket {
            name: "/*",
            limit: 20,
        }
    }
}

fn route_key(method: &Method, path: &str) -> String {
    static_bucket(method, path).name.to_owned()
}

fn clean_url_path(path: &str) -> String {
    let path = path.split('?').next().unwrap_or(path).trim();
    let path = format!("/{}", path.trim_start_matches('/').trim_end_matches('/'));
    if path == "/" {
        "/".to_string()
    } else {
        path
    }
}

fn matches_pattern(path: &str, pattern: &str) -> bool {
    let path_parts = path.trim_matches('/').split('/');
    let pattern_parts = pattern.trim_matches('/').split('/');

    path_parts
        .zip(pattern_parts)
        .all(|(part, pattern)| pattern.starts_with(':') || part == pattern)
        && path.trim_matches('/').split('/').count() == pattern.trim_matches('/').split('/').count()
}

fn header_str<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name)?.to_str().ok()
}

fn header_u32(headers: &HeaderMap, name: &str) -> Option<u32> {
    header_str(headers, name)?.parse().ok()
}

fn header_u64(headers: &HeaderMap, name: &str) -> Option<u64> {
    header_str(headers, name)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use reqwest::{
        header::{HeaderMap, HeaderValue},
        Method,
    };

    use super::{route_key, RateLimiter};

    #[test]
    fn message_routes_share_the_documented_message_bucket() {
        assert_eq!(
            route_key(&Method::POST, "/channels/abc/messages"),
            "POST /channels/:id/messages"
        );
        assert_eq!(
            route_key(&Method::POST, "/channels/def/messages"),
            "POST /channels/:id/messages"
        );
    }

    #[tokio::test]
    async fn acquire_reserves_capacity_before_requests_are_sent() {
        let limiter = RateLimiter::default();

        for _ in 0..10 {
            assert!(limiter
                .acquire(&Method::POST, "/channels/abc/messages")
                .await
                .is_zero());
        }

        let state = limiter.state.lock().await;
        let bucket = state
            .buckets
            .get("POST /channels/:id/messages")
            .expect("message bucket");
        assert_eq!(bucket.remaining, 0);
    }

    #[tokio::test]
    async fn response_headers_do_not_restore_concurrently_reserved_capacity() {
        let limiter = RateLimiter::default();
        let path = "/channels/abc/messages";

        for _ in 0..10 {
            assert!(limiter.acquire(&Method::POST, path).await.is_zero());
        }

        let mut headers = HeaderMap::new();
        headers.insert("X-RateLimit-Bucket", HeaderValue::from_static("messages"));
        headers.insert("X-RateLimit-Limit", HeaderValue::from_static("10"));
        headers.insert("X-RateLimit-Remaining", HeaderValue::from_static("9"));
        headers.insert("X-RateLimit-Reset-After", HeaderValue::from_static("10000"));
        limiter
            .update_from_headers(&Method::POST, path, &headers)
            .await;

        let state = limiter.state.lock().await;
        let bucket = state
            .buckets
            .get("POST /channels/:id/messages")
            .expect("message bucket");
        assert_eq!(bucket.remaining, 0);
    }
}
