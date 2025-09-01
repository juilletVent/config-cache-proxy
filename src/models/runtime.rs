use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct RuntimeInfo {
    /// 总请求数
    pub request_count: u64,
    /// 缓存命中数
    pub cache_hit_count: u64,
    /// 启动时间戳（毫秒）
    pub start_unix_time: u128,
}

#[derive(Debug)]
pub struct RuntimeStats {
    pub request_count: AtomicU64,
    pub cache_hit_count: AtomicU64,
    pub start_unix_time: u128,
}

impl RuntimeStats {
    pub fn new() -> Self {
        Self {
            request_count: AtomicU64::new(0),
            cache_hit_count: AtomicU64::new(0),
            start_unix_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        }
    }

    pub fn to_info(&self) -> RuntimeInfo {
        RuntimeInfo {
            request_count: self.request_count.load(Ordering::Relaxed),
            cache_hit_count: self.cache_hit_count.load(Ordering::Relaxed),
            start_unix_time: self.start_unix_time,
        }
    }

    pub fn increment_request_count(&self) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_cache_hit_count(&self) {
        self.cache_hit_count.fetch_add(1, Ordering::Relaxed);
    }
} 