use crate::{utils::errors::{AppError, AppResult}, AppState};
use axum::{
    extract::State,
    http::Uri,
    response::Html,
};
use std::fs::read_to_string;

pub async fn home_page(home_file_path: &str) -> AppResult<Html<String>> {
    let content = read_to_string(home_file_path)
        .map_err(|e| AppError::NotFound(format!("Home page not found: {}", e)))?;
    Ok(Html(content))
}

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
        (status = 500, description = "代理请求失败", body = crate::utils::errors::ErrorResponse)
    )
)]
pub async fn proxy_config_center(
    uri: Uri,
    State(app_state): State<AppState>,
) -> AppResult<String> {
    let uri_str = uri.to_string();
    
    // 增加请求计数
    app_state.runtime_stats.increment_request_count();

    match app_state.proxy_service.proxy_request(&uri_str).await? {
        Some(result) => {
            // 如果是从缓存返回的，增加缓存命中计数
            if result.from_cache {
                app_state.runtime_stats.increment_cache_hit_count();
            }
            Ok(result.content)
        }
        None => {
            // 非yml文件返回空字符串
            Ok(String::new())
        }
    }
} 