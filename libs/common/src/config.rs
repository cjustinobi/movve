use serde::Deserialize;

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
    pub auth_service_url: String,
    pub driver_service_url: String,
}

impl AppConfig {
    pub fn load() -> Result<Self, anyhow::Error> {
        let path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.yaml".into());
        let config_str = std::fs::read_to_string(&path)?;
        let config: Self = serde_yaml::from_str(&config_str)?;
        Ok(config)
    }
}