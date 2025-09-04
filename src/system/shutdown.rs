use std::time::Duration;

use tokio::signal;

/// 监听停机信号
pub async fn shutdown_signal() {
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
pub async fn cleanup_resources() {
    println!("🧹 正在清理资源...");
    // Redis 连接池会在析构时自动清理，给一些时间完成清理工作
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("✅ 资源清理完成");
}
