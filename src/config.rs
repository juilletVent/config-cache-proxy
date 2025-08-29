use std::{fs, sync::LazyLock};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemConfig {
    // 服务绑定地址
    pub server_address: String,
    // 服务端口
    pub server_port: u16,
    // Home 文件路径
    pub home_file_path: String,
    // 反向代理地址
    pub proxy_address: String,

    // Redis 配置
    pub redis: RedisConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RedisConfig {
    // Redis 地址
    pub address: String,
    // Redis 端口
    pub port: u16,
    // Redis 密码
    pub password: String,
    // 缓存过期时间
    pub cache_expire_time: u64,
}

pub static SYSTEM_CONFIG: LazyLock<SystemConfig> = LazyLock::new(|| {
    let config_str = fs::read_to_string("./config.yml").unwrap_or_else(|_| {
        eprintln!("Failed to read config.yml, please check the file path");
        std::process::exit(1);
    });

    serde_yml::from_str(&config_str).unwrap_or_else(|e| {
        eprintln!("Failed to parse config.yml: {}", e);
        std::process::exit(1);
    })
});
