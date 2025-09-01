pub mod config;
pub mod handlers;
pub mod models;
pub mod repositories;
pub mod services;
pub mod utils;

// 重新导出旧的redis_driver以保持兼容性
pub mod redis_driver;

use crate::{
    config::SystemConfig,
    handlers::{cache::clear_cache, health::get_runtime, proxy::proxy_config_center},
    models::{responses::ClearCacheResponse, runtime::RuntimeInfo},
    repositories::redis_repository::RedisRepository,
    services::{cache_service::CacheService, proxy_service::ProxyService},
    utils::errors::{AppResult, ErrorResponse},
    models::runtime::RuntimeStats,
};
use axum::{
    routing::{delete, get},
    Router,
};
use reqwest::Client;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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
            config.redis.password,
            config.redis.address,
            config.redis.port
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

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Config Cache Proxy API",
        description = "一个高性能的配置缓存代理服务，提供配置文件缓存和运行时监控功能",
        version = "0.1.0",
        contact(
            name = "API Support",
            email = "support@example.com"
        )
    ),
    paths(
        crate::handlers::health::get_runtime,
        crate::handlers::cache::clear_cache,
        crate::handlers::proxy::proxy_config_center
    ),
    components(
        schemas(RuntimeInfo, ClearCacheResponse, ErrorResponse)
    ),
    tags(
        (name = "monitoring", description = "监控和统计相关接口"),
        (name = "cache", description = "缓存管理相关接口"),
        (name = "proxy", description = "反向代理相关接口")
    )
)]
pub struct ApiDoc;

pub fn create_router(app_state: AppState) -> Router {
    let home_file_path = app_state.config.home_file_path.clone();
    
    Router::new()
        .route("/", get(move || async move {
            crate::handlers::proxy::home_page(&home_file_path).await
        }))
        .route("/get-runtime", get(get_runtime))
        .route("/clear-cache", delete(clear_cache))
        .route("/{*all}", get(proxy_config_center))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(app_state)
}
