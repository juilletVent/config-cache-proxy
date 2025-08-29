use anyhow::{Context, Result};
use deadpool_redis::{Config, Connection, Pool, Runtime, redis::cmd};
use redis::AsyncCommands;
use urlencoding::encode;

use crate::config::SYSTEM_CONFIG;

pub struct RedisCore {
    pool: Option<Pool>,
    url: String,
}

const CACHE_PREFIX: &str = "config_cache:";

impl RedisCore {
    pub fn new() -> Self {
        Self {
            pool: None,
            url: format!(
                "redis://:{}@{}:{}/0",
                SYSTEM_CONFIG.redis.password, SYSTEM_CONFIG.redis.address, SYSTEM_CONFIG.redis.port
            ),
        }
    }
    /**
     * 初始化Redis连接池
     */
    pub fn init_redis_pool(mut self) -> anyhow::Result<Self> {
        let cfg = Config::from_url(&self.url);
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .context("Failed to create Redis pool")?; // 使用 anyhow 包装错误，给出更明确的错误信息
        self.pool = Some(pool);
        Ok(self)
    }

    pub fn get_redis_pool(&self) -> Option<&Pool> {
        self.pool.as_ref()
    }

    pub async fn get(&self, key: &str) -> Result<String, anyhow::Error> {
        let mut client = self
            .pool
            .as_ref()
            .unwrap()
            .get()
            .await
            .context("获取Redis连接失败")?;
        let value: String = client
            .get(format!("{CACHE_PREFIX}{}", encode(key)))
            .await
            .context("获取Redis值失败")?;
        Ok(value)
    }

    pub async fn set(&self, key: &str, value: &str, expire: u64) -> Result<(), anyhow::Error> {
        let mut client = self
            .pool
            .as_ref()
            .unwrap()
            .get()
            .await
            .context("获取Redis连接失败")?;
        let _: () = client
            .set_ex(format!("{CACHE_PREFIX}{}", encode(key)), value, expire)
            .await
            .context("设置Redis值失败")?;
        Ok(())
    }

    pub async fn delete_all(&self) -> Result<usize, anyhow::Error> {
        let mut client = self
            .pool
            .as_ref()
            .unwrap()
            .get()
            .await
            .context("获取Redis连接失败")?;

        let pattern = format!("{CACHE_PREFIX}*");
        let mut cursor = 0u64;
        let mut total_deleted = 0usize;

        loop {
            let (next_cursor, keys): (u64, Vec<String>) = cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut client)
                .await
                .context("获取Redis值失败")?;

            if !keys.is_empty() {
                let deleted_count = self.delete_keys(&mut client, &keys).await?;
                total_deleted += deleted_count;
            }

            cursor = next_cursor;
            if cursor == 0 {
                // 如果游标为0，表示没有更多数据了
                break;
            }
        }

        Ok(total_deleted)
    }

    /// 批量删除指定的键列表
    async fn delete_keys(
        &self,
        conn: &mut Connection,
        keys: &[String],
    ) -> Result<usize, anyhow::Error> {
        if keys.is_empty() {
            return Ok(0);
        }

        // 使用 DEL 命令批量删除
        let deleted_count: usize = cmd("DEL").arg(keys).query_async(conn).await?;

        Ok(deleted_count)
    }

    pub async fn ping(&self) -> Result<(), anyhow::Error> {
        let mut client = self
            .pool
            .as_ref()
            .unwrap()
            .get()
            .await
            .context("获取Redis连接失败")?;

        let _: () = client.ping().await.context("Redis ping 失败")?;
        Ok(())
    }
}
