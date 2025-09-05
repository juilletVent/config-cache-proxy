use std::sync::Arc;

use crate::{
    models::runtime::RuntimeStats,
    repositories::redis_repository::RedisRepository,
    services::{cache_service::CacheService, proxy_service::ProxyService},
    system::SystemConfig,
    utils::errors::AppResult,
};
use reqwest::Client;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<SystemConfig>,
    pub runtime_stats: Arc<RuntimeStats>,
    pub cache_service: Arc<CacheService>,
    pub proxy_service: Arc<ProxyService>,
}

impl AppState {
    pub async fn new(config: SystemConfig) -> AppResult<Self> {
        let config = Arc::new(config);
        let runtime_stats = Arc::new(RuntimeStats::new());

        // 创建Redis URL
        let redis_url = format!(
            "redis://:{}@{}:{}/0",
            config.redis.password, config.redis.address, config.redis.port
        );

        // 创建Redis Repository
        let redis_repo = Arc::new(RedisRepository::new(&redis_url)?);

        // 测试Redis连接
        redis_repo.ping().await?;

        // 创建服务
        let cache_service = Arc::new(CacheService::new(redis_repo));
        let http_client = Client::new();
        let proxy_service = Arc::new(ProxyService::new(
            cache_service.clone(),
            http_client,
            config.proxy_address.clone(),
            config.redis.cache_expire_time,
        ));

        Ok(Self {
            config,
            runtime_stats,
            cache_service,
            proxy_service,
        })
    }
}
