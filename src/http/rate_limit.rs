use {
    reqwest::{header::HeaderMap, Method},
    serde::Deserialize,
    std::{collections::HashMap, time::Duration},
    tokio::{
        sync::Mutex,
        time::{sleep, Instant},
    },
};

/// Tracks Stoat REST rate-limit buckets and waits before requests when needed.
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
    routes: HashMap<String, String>,
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
    /// Wait until the route's known bucket has capacity.
    pub async fn wait(&self, method: &Method, path: &str) {
        let route = route_key(method, path);

        loop {
            let sleep_for = {
                let state = self.state.lock().await;
                state
                    .routes
                    .get(&route)
                    .or(Some(&route))
                    .and_then(|bucket| state.buckets.get(bucket))
                    .and_then(|bucket| bucket.delay_until_available())
            };

            match sleep_for {
                Some(delay) if !delay.is_zero() => sleep(delay).await,
                _ => return,
            }
        }
    }

    /// Update tracked bucket state from Stoat rate-limit headers.
    pub async fn update_from_headers(&self, method: &Method, path: &str, headers: &HeaderMap) {
        let Some(bucket_id) = header_str(headers, "X-RateLimit-Bucket").map(ToOwned::to_owned)
        else {
            return;
        };

        let limit = header_u32(headers, "X-RateLimit-Limit");
        let remaining = header_u32(headers, "X-RateLimit-Remaining");
        let reset_after = header_u64(headers, "X-RateLimit-Reset-After");

        let mut state = self.state.lock().await;
        let route = route_key(method, path);
        state.routes.insert(route, bucket_id.clone());

        let previous = state.buckets.get(&bucket_id).cloned();
        let fallback = static_bucket(method, path);
        let limit = limit
            .or_else(|| fallback.map(|bucket| bucket.limit))
            .or(previous.as_ref().map(|bucket| bucket.limit))
            .unwrap_or(1);
        let remaining = remaining
            .or(previous.as_ref().map(|bucket| bucket.remaining))
            .unwrap_or(limit);
        let reset_after = reset_after
            .or_else(|| {
                previous.as_ref().map(|bucket| {
                    bucket
                        .reset_at
                        .saturating_duration_since(Instant::now())
                        .as_millis() as u64
                })
            })
            .unwrap_or(10_000);

        state.buckets.insert(
            bucket_id,
            BucketState {
                limit,
                remaining,
                reset_at: Instant::now() + Duration::from_millis(reset_after),
            },
        );
    }

    /// Mark the route as rate-limited for the provided retry interval.
    pub async fn update_retry_after(&self, method: &Method, path: &str, retry_after_ms: u64) {
        let route = route_key(method, path);
        let fallback = static_bucket(method, path);
        let bucket_id = fallback
            .as_ref()
            .map(|bucket| bucket.name.to_owned())
            .unwrap_or_else(|| route.clone());

        let mut state = self.state.lock().await;
        let bucket_id = state.routes.get(&route).cloned().unwrap_or(bucket_id);
        state.routes.insert(route, bucket_id.clone());
        state.buckets.insert(
            bucket_id,
            BucketState {
                limit: fallback.map(|bucket| bucket.limit).unwrap_or(1),
                remaining: 0,
                reset_at: Instant::now() + Duration::from_millis(retry_after_ms),
            },
        );
    }
}

impl BucketState {
    fn delay_until_available(&self) -> Option<Duration> {
        if self.remaining > 0 {
            return None;
        }

        let now = Instant::now();
        if now >= self.reset_at {
            return None;
        }

        Some(self.reset_at - now)
    }
}

#[derive(Debug, Clone, Copy)]
struct StaticBucket {
    name: &'static str,
    limit: u32,
}

fn static_bucket(method: &Method, path: &str) -> Option<StaticBucket> {
    let path = normalise_path(path);

    if method.as_str() == Method::PATCH.as_str() && matches_pattern(&path, "/users/:id") {
        return Some(StaticBucket {
            name: "PATCH /users/:id",
            limit: 2,
        });
    }

    if method.as_str() == Method::POST.as_str() && matches_pattern(&path, "/channels/:id/messages")
    {
        return Some(StaticBucket {
            name: "POST /channels/:id/messages",
            limit: 10,
        });
    }

    if method.as_str() == Method::DELETE.as_str() && path.starts_with("/auth") {
        return Some(StaticBucket {
            name: "DELETE /auth",
            limit: 255,
        });
    }

    let bucket = if matches_pattern(&path, "/users/:id/default_avatar") {
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
    };

    Some(bucket)
}

fn route_key(method: &Method, path: &str) -> String {
    static_bucket(method, path)
        .map(|bucket| bucket.name.to_owned())
        .unwrap_or_else(|| format!("{} {}", method.as_str(), normalise_path(path)))
}

fn normalise_path(path: &str) -> String {
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
