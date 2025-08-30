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
    routing::get,
};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct RuntimeInfo {
    request_count: u64,
    cache_hit_count: u64,
    start_unix_time: u128,
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

pub fn register_routes(mut app: Router) -> Router {
    app = app
        .route("/", get(home_page))
        .route("/get-runtime", get(get_runtime))
        .route("/clear-cache", get(clear_cache))
        .route("/{*all}", get(proxy_config_center));
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

async fn get_runtime() -> Json<RuntimeInfo> {
    // 返回运行信息的JSON
    Json(RUNTIME_STATS.to_info())
}

async fn clear_cache() -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    let total_deleted = get_redis_manager()
        .await
        .delete_all()
        .await
        .unwrap_or_else(|err| {
            eprintln!("清理缓存失败: {}", err);
            0
        });

    let response = serde_json::json!({
        "success": true,
        "message": format!("成功清理了 {} 个缓存条目", total_deleted),
        "deleted_count": total_deleted
    });

    Ok(Json(response))
}

static YML_EXT_SUFFIX: &str = ".yml";

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
