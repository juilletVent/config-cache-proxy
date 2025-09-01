use crate::utils::errors::{AppError, AppResult};
use deadpool_redis::{Config, Connection, Pool, Runtime, redis::cmd};
use redis::AsyncCommands;
use std::sync::Arc;
use urlencoding::encode;

const CACHE_PREFIX: &str = "config_cache:";

#[derive(Clone)]
pub struct RedisRepository {
    pool: Arc<Pool>,
}

impl RedisRepository {
    pub fn new(redis_url: &str) -> AppResult<Self> {
        let cfg = Config::from_url(redis_url);
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| AppError::Config(format!("Failed to create Redis pool: {}", e)))?;
        
        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    pub async fn ping(&self) -> AppResult<()> {
        let mut conn = self.get_connection().await?;
        let _: () = conn.ping().await.map_err(AppError::RedisCommand)?;
        Ok(())
    }

    pub async fn get(&self, key: &str) -> AppResult<Option<String>> {
        let mut conn = self.get_connection().await?;
        let cache_key = format!("{}{}", CACHE_PREFIX, encode(key));
        
        let result: Option<String> = conn
            .get(&cache_key)
            .await
            .map_err(AppError::RedisCommand)?;
        
        Ok(result)
    }

    pub async fn set(&self, key: &str, value: &str, expire_seconds: u64) -> AppResult<()> {
        let mut conn = self.get_connection().await?;
        let cache_key = format!("{}{}", CACHE_PREFIX, encode(key));
        
        let _: () = conn.set_ex(&cache_key, value, expire_seconds)
            .await
            .map_err(AppError::RedisCommand)?;
        
        Ok(())
    }

    pub async fn delete_all(&self) -> AppResult<usize> {
        let mut conn = self.get_connection().await?;
        let pattern = format!("{}*", CACHE_PREFIX);
        let mut cursor = 0u64;
        let mut total_deleted = 0usize;

        loop {
            let (next_cursor, keys): (u64, Vec<String>) = cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await
                .map_err(AppError::RedisCommand)?;

            if !keys.is_empty() {
                let deleted_count = self.delete_keys(&mut conn, &keys).await?;
                total_deleted += deleted_count;
            }

            cursor = next_cursor;
            if cursor == 0 {
                break;
            }
        }

        Ok(total_deleted)
    }

    async fn get_connection(&self) -> AppResult<Connection> {
        self.pool.get().await.map_err(AppError::Redis)
    }

    async fn delete_keys(&self, conn: &mut Connection, keys: &[String]) -> AppResult<usize> {
        if keys.is_empty() {
            return Ok(0);
        }

        let deleted_count: usize = cmd("DEL")
            .arg(keys)
            .query_async(conn)
            .await
            .map_err(AppError::RedisCommand)?;

        Ok(deleted_count)
    }
} 