use crate::repositories::redis_repository::RedisRepository;
use crate::utils::errors::AppResult;
use std::sync::Arc;

#[derive(Clone)]
pub struct CacheService {
    redis_repo: Arc<RedisRepository>,
}

impl CacheService {
    pub fn new(redis_repo: Arc<RedisRepository>) -> Self {
        Self { redis_repo }
    }

    pub async fn get(&self, key: &str) -> AppResult<Option<String>> {
        self.redis_repo.get(key).await
    }

    pub async fn set(&self, key: &str, value: &str, expire_seconds: u64) -> AppResult<()> {
        self.redis_repo.set(key, value, expire_seconds).await
    }

    pub async fn clear_all(&self) -> AppResult<usize> {
        self.redis_repo.delete_all().await
    }
} 