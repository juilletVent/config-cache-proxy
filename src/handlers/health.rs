use crate::{models::runtime::RuntimeInfo, AppState};
use axum::{extract::State, response::Json};

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
pub async fn get_runtime(State(app_state): State<AppState>) -> Json<RuntimeInfo> {
    Json(app_state.runtime_stats.to_info())
} 