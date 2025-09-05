use std::fs;

use crate::utils::errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl SystemConfig {
    pub fn load_from_file(path: &str) -> AppResult<Self> {
        let config_str = fs::read_to_string(path).map_err(|e| {
            AppError::Config(format!("Failed to read config file '{}': {}", path, e))
        })?;

        let config: SystemConfig = serde_yml::from_str(&config_str).map_err(|e| {
            AppError::Config(format!("Failed to parse config file '{}': {}", path, e))
        })?;

        Ok(config)
    }
}
