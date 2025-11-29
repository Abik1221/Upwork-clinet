use anyhow::Result;
use dotenv::dotenv;
use std::env;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    // OpenAI Configuration
    pub openai_api_key: String,
    pub openai_chat_model: String,
    pub openai_embedding_model: String,

    // Server Configuration
    pub server_host: String,
    pub server_port: u16,

    // Vector Database Configuration
    pub qdrant_path: String,

    // Rate Limiting Configuration
    pub max_requests_per_minute: u32,
    pub max_requests_per_hour: u32,

    // Circuit Breaker Configuration
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_timeout_seconds: u64,

    // PDF Processing Configuration
    pub max_pdf_size_mb: u64,
    pub chunk_size_tokens: usize,
    pub chunk_overlap_tokens: usize,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        // Load .env file if it exists
        dotenv().ok();

        Ok(Config {
            // OpenAI Configuration
            openai_api_key: env::var("OPENAI_API_KEY")
                .expect("OPENAI_API_KEY must be set in .env file"),
            openai_chat_model: env::var("OPENAI_CHAT_MODEL")
                .unwrap_or_else(|_| "gpt-4o-mini".to_string()),
            openai_embedding_model: env::var("OPENAI_EMBEDDING_MODEL")
                .unwrap_or_else(|_| "text-embedding-3-small".to_string()),

            // Server Configuration
            server_host: env::var("SERVER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("SERVER_PORT must be a valid port number"),

            // Vector Database Configuration
            qdrant_path: env::var("QDRANT_PATH")
                .unwrap_or_else(|_| "./qdrant_storage".to_string()),

            // Rate Limiting Configuration
            max_requests_per_minute: env::var("MAX_REQUESTS_PER_MINUTE")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .expect("MAX_REQUESTS_PER_MINUTE must be a number"),
            max_requests_per_hour: env::var("MAX_REQUESTS_PER_HOUR")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .expect("MAX_REQUESTS_PER_HOUR must be a number"),

            // Circuit Breaker Configuration
            circuit_breaker_threshold: env::var("CIRCUIT_BREAKER_THRESHOLD")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("CIRCUIT_BREAKER_THRESHOLD must be a number"),
            circuit_breaker_timeout_seconds: env::var("CIRCUIT_BREAKER_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .expect("CIRCUIT_BREAKER_TIMEOUT_SECONDS must be a number"),

            // PDF Processing Configuration
            max_pdf_size_mb: env::var("MAX_PDF_SIZE_MB")
                .unwrap_or_else(|_| "50".to_string())
                .parse()
                .expect("MAX_PDF_SIZE_MB must be a number"),
            chunk_size_tokens: env::var("CHUNK_SIZE_TOKENS")
                .unwrap_or_else(|_| "512".to_string())
                .parse()
                .expect("CHUNK_SIZE_TOKENS must be a number"),
            chunk_overlap_tokens: env::var("CHUNK_OVERLAP_TOKENS")
                .unwrap_or_else(|_| "50".to_string())
                .parse()
                .expect("CHUNK_OVERLAP_TOKENS must be a number"),
        })
    }

    /// Validate that all required configuration is present
    pub fn validate(&self) -> Result<()> {
        if self.openai_api_key.is_empty() || self.openai_api_key == "sk-your-api-key-here" {
            anyhow::bail!("OPENAI_API_KEY must be set to a valid API key");
        }

        if self.server_port == 0 {
            anyhow::bail!("SERVER_PORT must be a valid port number");
        }

        log::info!("Configuration loaded successfully");
        log::info!("  Server: {}:{}", self.server_host, self.server_port);
        log::info!("  Chat Model: {}", self.openai_chat_model);
        log::info!("  Embedding Model: {}", self.openai_embedding_model);
        log::info!(
            "  Rate Limits: {}/min, {}/hour",
            self.max_requests_per_minute, self.max_requests_per_hour
        );

        Ok(())
    }
}
