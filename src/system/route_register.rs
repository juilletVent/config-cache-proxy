use axum::Router;
use axum::routing::method_routing::*;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::cache::clear_cache;
use crate::handlers::health::get_runtime;
use crate::handlers::proxy::proxy_config_center;
use crate::models::{responses::ClearCacheResponse, runtime::RuntimeInfo};
use crate::system::AppState;
use crate::utils::errors::ErrorResponse;

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
        .route(
            "/",
            get(move || async move { crate::handlers::proxy::home_page(&home_file_path).await }),
        )
        .route("/get-runtime", get(get_runtime))
        .route("/clear-cache", delete(clear_cache))
        .route("/{*all}", get(proxy_config_center))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(app_state)
}
