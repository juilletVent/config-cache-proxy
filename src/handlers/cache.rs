use crate::{models::responses::ClearCacheResponse, system::AppState, utils::errors::AppResult};
use axum::{extract::State, response::Json};

#[utoipa::path(
    delete,
    path = "/clear-cache",
    tag = "cache",
    summary = "清理所有缓存",
    description = "清理Redis中的所有缓存条目，返回清理的条目数量",
    responses(
        (status = 200, description = "成功清理缓存", body = ClearCacheResponse),
        (status = 500, description = "内部服务器错误", body = crate::utils::errors::ErrorResponse)
    )
)]
pub async fn clear_cache(State(app_state): State<AppState>) -> AppResult<Json<ClearCacheResponse>> {
    let deleted_count = app_state.cache_service.clear_all().await?;

    let response = ClearCacheResponse {
        success: true,
        message: format!("成功清理了 {} 个缓存条目", deleted_count),
        deleted_count: deleted_count as u64,
    };

    Ok(Json(response))
}
