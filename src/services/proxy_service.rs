use crate::services::cache_service::CacheService;
use crate::utils::errors::{AppError, AppResult};
use reqwest::Client;
use std::sync::Arc;

static YML_EXT_SUFFIX: &str = ".yml";

#[derive(Clone)]
pub struct ProxyService {
    cache_service: Arc<CacheService>,
    http_client: Client,
    proxy_base_url: String,
    cache_expire_seconds: u64,
}

pub struct ProxyResult {
    pub content: String,
    pub from_cache: bool,
}

impl ProxyService {
    pub fn new(
        cache_service: Arc<CacheService>,
        http_client: Client,
        proxy_base_url: String,
        cache_expire_seconds: u64,
    ) -> Self {
        Self {
            cache_service,
            http_client,
            proxy_base_url,
            cache_expire_seconds,
        }
    }

    pub async fn proxy_request(&self, path: &str) -> AppResult<Option<ProxyResult>> {
        // 只处理 yml 文件
        if !path.ends_with(YML_EXT_SUFFIX) {
            return Ok(None);
        }

        let url = format!("{}{}", self.proxy_base_url, path);

        // 检查缓存
        if let Some(cached_response) = self.cache_service.get(&url).await? {
            return Ok(Some(ProxyResult {
                content: cached_response,
                from_cache: true,
            }));
        }

        // 发送HTTP请求
        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(AppError::HttpClient)?;

        if !response.status().is_success() {
            return Err(AppError::Proxy(format!(
                "Upstream returned status: {} for URL: {}",
                response.status(),
                url
            )));
        }

        let response_text = response.text().await.map_err(AppError::HttpClient)?;

        // 缓存响应
        if let Err(e) = self
            .cache_service
            .set(&url, &response_text, self.cache_expire_seconds)
            .await
        {
            // 缓存失败不应该影响主要业务流程，只记录错误
            tracing::warn!("Failed to cache response for URL {}: {}", url, e);
        }

        Ok(Some(ProxyResult {
            content: response_text,
            from_cache: false,
        }))
    }
} 