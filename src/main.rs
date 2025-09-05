use config_cache_proxy::system::{AppState, SystemConfig, create_router, shutdown_signal};

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
