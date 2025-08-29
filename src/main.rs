use axum::Router;
use config_cache_proxy::{
    config::SYSTEM_CONFIG, redis_driver::get_redis_manager, route::register_routes,
};
use std::time::Duration;
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 正在启动 Config Cache Proxy 服务...");
    // 测试Redis连接（这会触发异步初始化和ping测试）
    println!("🔗 正在初始化Redis连接池并测试连接...");
    let _redis = get_redis_manager().await;
    println!("✅ Redis连接池初始化成功，ping测试通过！");

    let app = register_routes(Router::new());

    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        SYSTEM_CONFIG.server_address, SYSTEM_CONFIG.server_port
    ))
    .await
    .unwrap();

    println!(
        "服务器已启动，监听地址: {}:{}",
        SYSTEM_CONFIG.server_address, SYSTEM_CONFIG.server_port
    );
    println!("按 Ctrl+C 或发送 SIGTERM 信号进行停机");

    // 启动服务器并支持优雅停机
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    println!("服务器已停机");
    Ok(())
}

/// 监听停机信号
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!("\n收到 Ctrl+C 信号，开始停机...");
        },
        _ = terminate => {
            println!("收到 SIGTERM 信号，开始停机...");
        },
    }

    // 执行清理工作
    cleanup_resources().await;
}

/// 清理资源
async fn cleanup_resources() {
    println!("正在清理资源...");
    // Redis 连接池会在析构时自动清理，无需手动处理，给一些时间完成清理工作
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("资源清理完成");
}
