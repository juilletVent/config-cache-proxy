use std::{
    fs::read_to_string,
    sync::{
        LazyLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{config::SYSTEM_CONFIG, redis_driver::get_redis_manager};
use axum::{
    Router,
    http::Uri,
    response::{Html, Json},
    routing::delete,
    routing::get,
};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Serialize, Deserialize, Clone, ToSchema)]
struct RuntimeInfo {
    /// 总请求数
    request_count: u64,
    /// 缓存命中数
    cache_hit_count: u64,
    /// 启动时间戳（毫秒）
    start_unix_time: u128,
}

#[derive(Serialize, Deserialize, ToSchema)]
struct ClearCacheResponse {
    /// 操作是否成功
    success: bool,
    /// 响应消息
    message: String,
    /// 删除的缓存条目数量
    deleted_count: u64,
}

#[derive(Serialize, Deserialize, ToSchema)]
struct ErrorResponse {
    /// 错误消息
    error: String,
}

struct RuntimeStats {
    request_count: AtomicU64,
    cache_hit_count: AtomicU64,
    start_unix_time: u128,
}

impl RuntimeStats {
    fn to_info(&self) -> RuntimeInfo {
        RuntimeInfo {
            request_count: self.request_count.load(Ordering::Relaxed),
            cache_hit_count: self.cache_hit_count.load(Ordering::Relaxed),
            start_unix_time: self.start_unix_time,
        }
    }

    fn increment_request_count(&self) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
    }

    fn increment_cache_hit_count(&self) {
        self.cache_hit_count.fetch_add(1, Ordering::Relaxed);
    }
}

static RUNTIME_STATS: LazyLock<RuntimeStats> = LazyLock::new(|| RuntimeStats {
    request_count: AtomicU64::new(0),
    cache_hit_count: AtomicU64::new(0),
    start_unix_time: SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis(),
});

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
        get_runtime,
        clear_cache,
        proxy_config_center
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

pub fn register_routes(mut app: Router) -> Router {
    app = app
        .route("/", get(home_page))
        .route("/get-runtime", get(get_runtime))
        .route("/clear-cache", delete(clear_cache))
        .route("/{*all}", get(proxy_config_center))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));
    app
}

async fn home_page() -> Result<Html<String>, (StatusCode, &'static str)> {
    if let Ok(content) = read_to_string(&SYSTEM_CONFIG.home_file_path) {
        Ok(Html(content))
    } else {
        // 读取文件失败
        Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"))
    }
}

#[utoipa::path(
    get,
    path = "/get-runtime",
    tag = "monitoring",
    summary = "获取运行时统计信息",
    description = "返回服务的运行时统计信息，包括请求总数、缓存命中数和启动时间",
    responses(
        (status = 200, description = "成功返回运行时信息", body = RuntimeInfo)
    )
)]
async fn get_runtime() -> Json<RuntimeInfo> {
    // 返回运行信息的JSON
    Json(RUNTIME_STATS.to_info())
}

#[utoipa::path(
    delete,
    path = "/clear-cache",
    tag = "cache",
    summary = "清理所有缓存",
    description = "清理Redis中的所有缓存条目，返回清理的条目数量",
    responses(
        (status = 200, description = "成功清理缓存", body = ClearCacheResponse),
        (status = 500, description = "内部服务器错误", body = ErrorResponse)
    )
)]
async fn clear_cache() -> Result<Json<ClearCacheResponse>, (StatusCode, Json<ErrorResponse>)> {
    let total_deleted = get_redis_manager()
        .await
        .delete_all()
        .await
        .unwrap_or_else(|err| {
            eprintln!("清理缓存失败: {}", err);
            0
        });

    let response = ClearCacheResponse {
        success: true,
        message: format!("成功清理了 {} 个缓存条目", total_deleted),
        deleted_count: total_deleted as u64,
    };

    Ok(Json(response))
}

static YML_EXT_SUFFIX: &str = ".yml";

#[utoipa::path(
    get,
    path = "/{path}",
    tag = "proxy",
    summary = "代理配置中心请求",
    description = "代理对配置中心的请求，支持缓存机制。只处理以.yml结尾的文件请求",
    params(
        ("path" = String, description = "要代理的配置文件路径")
    ),
    responses(
        (status = 200, description = "成功返回配置文件内容", body = String),
        (status = 500, description = "代理请求失败", body = ErrorResponse)
    )
)]
async fn proxy_config_center(uri: Uri) -> Result<String, (StatusCode, String)> {
    // println!("REQUEST_URI: {}", uri.to_string());

    let uri_str = uri.to_string();
    // 忽略非yml文件的请求
    if !uri_str.ends_with(YML_EXT_SUFFIX) {
        return Ok("".to_string());
    }

    // 获取请求的url
    let url = format!("{}{}", SYSTEM_CONFIG.proxy_address, uri.to_string());

    // 增加请求计数
    RUNTIME_STATS.increment_request_count();

    // 检查缓存
    let cached_response = get_redis_manager()
        .await
        .get(&url)
        .await
        .unwrap_or_else(|_| "".to_string());
    if !cached_response.is_empty() {
        // 增加缓存命中计数
        RUNTIME_STATS.increment_cache_hit_count();
        return Ok(cached_response);
    }

    // 发送请求
    let response = Client::new().get(url.to_string()).send().await;
    // 返回响应
    if let Ok(response) = response {
        let response_text = response.text().await.unwrap();
        // 缓存响应
        get_redis_manager()
            .await
            .set(&url, &response_text, SYSTEM_CONFIG.redis.cache_expire_time)
            .await
            .unwrap_or_else(|err| {
                eprintln!("Cache set failed, url: {}, error: {}", url, err);
            });
        Ok(response_text)
    } else {
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to send request, url: {}", url),
        ))
    }
}
