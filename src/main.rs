use config_cache_proxy::{AppState, config::SystemConfig, create_router};
use std::time::Duration;
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 正在启动 Config Cache Proxy 服务...");

    // 加载配置
    let config = SystemConfig::load_from_file("./config.yml")
        .map_err(|e| anyhow::anyhow!("配置加载失败: {}", e))?;

    println!("📋 配置加载成功");

    // 初始化应用状态（包含所有依赖）
    println!("🔗 正在初始化应用状态和Redis连接池...");
    let app_state = AppState::new(config.clone())
        .await
        .map_err(|e| anyhow::anyhow!("应用状态初始化失败: {}", e))?;

    println!("✅ 应用状态初始化成功，Redis连接测试通过！");

    // 创建路由
    let app = create_router(app_state);

    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", config.server_address, config.server_port))
            .await
            .unwrap();

    println!(
        "🌟 服务器已启动，监听地址: {}:{}",
        config.server_address, config.server_port
    );
    println!(
        "📚 API文档地址: http://{}:{}/swagger-ui",
        config.server_address, config.server_port
    );
    println!("🛑 按 Ctrl+C 或发送 SIGTERM 信号进行优雅停机");

    // 启动服务器并支持优雅停机
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    println!("👋 服务器已优雅停机");
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
            println!("\n🛑 收到 Ctrl+C 信号，开始优雅停机...");
        },
        _ = terminate => {
            println!("🛑 收到 SIGTERM 信号，开始优雅停机...");
        },
    }

    // 执行清理工作
    cleanup_resources().await;
}

/// 清理资源
async fn cleanup_resources() {
    println!("🧹 正在清理资源...");
    // Redis 连接池会在析构时自动清理，给一些时间完成清理工作
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("✅ 资源清理完成");
}
