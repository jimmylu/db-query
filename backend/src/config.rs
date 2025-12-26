use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub llm: LlmConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmConfig {
    pub gateway_url: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub style: String,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let mut builder = config::Config::builder()
            .set_default("database.url", "./metadata.db")?
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 3000)?
            .set_default("llm.gateway_url", "http://localhost:8080")?
            .set_default("logging.level", "info")?
            .set_default("logging.style", "auto")?;

        // Load from environment variables
        if let Ok(database_url) = env::var("DATABASE_URL") {
            builder = builder.set_override("database.url", database_url)?;
        }

        if let Ok(host) = env::var("HOST") {
            builder = builder.set_override("server.host", host)?;
        }

        if let Ok(port) = env::var("PORT") {
            builder = builder.set_override("server.port", port.parse::<u16>().unwrap_or(3000))?;
        }

        if let Ok(gateway_url) = env::var("LLM_GATEWAY_URL") {
            builder = builder.set_override("llm.gateway_url", gateway_url)?;
        }

        if let Ok(api_key) = env::var("LLM_API_KEY") {
            builder = builder.set_override("llm.api_key", Some(api_key))?;
        }

        if let Ok(log_level) = env::var("RUST_LOG") {
            builder = builder.set_override("logging.level", log_level)?;
        }

        if let Ok(log_style) = env::var("RUST_LOG_STYLE") {
            builder = builder.set_override("logging.style", log_style)?;
        }

        // Try to load from .env file
        let _ = dotenv::dotenv();

        builder.build()?.try_deserialize()
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        // Clear environment variables for this test
        env::remove_var("DATABASE_URL");
        env::remove_var("HOST");
        env::remove_var("PORT");
        
        let config = Config::from_env();
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.server.host, "0.0.0.0");
    }
}

