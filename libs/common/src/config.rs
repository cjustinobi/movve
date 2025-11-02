use serde::Deserialize;
use anyhow::Context;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub jwt: JwtConfig,
    pub services: ServicesConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_hours: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServicesConfig {
    pub auth_service_host: String,
    pub driver_service_host: String,
    pub auth_service_url: String,
    pub driver_service_url: String,
    pub auth_service_port: u16,
    pub driver_service_port: u16,
}

impl AppConfig {
    pub fn load() -> Result<Self, anyhow::Error> {

        dotenvy::dotenv().ok();

        let config = AppConfig {
            server: ServerConfig {
                host: std::env::var("SERVER_HOST")
                    .unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: std::env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .context("Failed to parse SERVER_PORT")?,
            },
            jwt: JwtConfig {
                secret: std::env::var("JWT_SECRET")
                    .context("JWT_SECRET must be set in environment")?,
                expiration_hours: std::env::var("JWT_EXPIRATION_HOURS")
                    .unwrap_or_else(|_| "24".to_string())
                    .parse()
                    .context("Failed to parse JWT_EXPIRATION_HOURS")?,
            },
            services: ServicesConfig {
                auth_service_host: std::env::var("AUTH_SERVICE_HOST")
                    .unwrap_or_else(|_| "127.0.0.1".to_string()),
                driver_service_host: std::env::var("DRIVER_SERVICE_HOST")
                    .unwrap_or_else(|_| "127.0.0.1".to_string()),
                auth_service_url: std::env::var("AUTH_SERVICE_URL")
                    .unwrap_or_else(|_| "http://localhost:8001".to_string()),
                driver_service_url: std::env::var("DRIVER_SERVICE_URL")
                    .unwrap_or_else(|_| "http://localhost:8002".to_string()),
                auth_service_port: std::env::var("AUTH_SERVICE_PORT")
                    .unwrap_or_else(|_| "8001".to_string())
                    .parse()
                    .context("Failed to parse AUTH_SERVICE_PORT")?,
                driver_service_port: std::env::var("DRIVER_SERVICE_PORT")
                    .unwrap_or_else(|_| "8002".to_string())
                    .parse()
                    .context("Failed to parse DRIVER_SERVICE_PORT")?,
            },
        };

        Ok(config)
    }
}