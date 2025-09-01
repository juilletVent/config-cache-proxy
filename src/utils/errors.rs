use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Redis error: {0}")]
    Redis(#[from] deadpool_redis::PoolError),
    
    #[error("Redis command error: {0}")]
    RedisCommand(#[from] redis::RedisError),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Cache operation failed: {0}")]
    Cache(String),
    
    #[error("Proxy request failed: {0}")]
    Proxy(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl AppError {
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::Redis(_) | AppError::RedisCommand(_) => "REDIS_ERROR",
            AppError::Config(_) => "CONFIG_ERROR", 
            AppError::HttpClient(_) => "HTTP_CLIENT_ERROR",
            AppError::Io(_) => "IO_ERROR",
            AppError::Serialization(_) => "SERIALIZATION_ERROR",
            AppError::Cache(_) => "CACHE_ERROR",
            AppError::Proxy(_) => "PROXY_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
            AppError::NotFound(_) => "NOT_FOUND",
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Config(_) => StatusCode::BAD_REQUEST,
            AppError::Proxy(_) => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let error_response = ErrorResponse {
            error: self.to_string(),
            code: self.error_code().to_string(),
            details: None,
        };

        let status = self.status_code();
        
        tracing::error!("Application error: {} (code: {})", self, self.error_code());
        
        (status, Json(error_response)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>; 