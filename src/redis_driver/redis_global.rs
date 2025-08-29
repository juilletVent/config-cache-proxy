use tokio::sync::OnceCell;

use crate::redis_driver::RedisCore;

static REDIS_MANAGER: OnceCell<RedisCore> = OnceCell::const_new();

pub async fn get_redis_manager() -> &'static RedisCore {
    REDIS_MANAGER
        .get_or_init(|| async {
            let redis_core = RedisCore::new().init_redis_pool().unwrap();

            // 进行测试，如果不通过，直接退出
            redis_core.ping().await.unwrap_or_else(|err| {
                eprintln!("Redis 连接测试失败，请检查Redis连接配置: {}", err);
                std::process::exit(1);
            });

            redis_core
        })
        .await
}
