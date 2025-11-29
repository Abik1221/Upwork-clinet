mod config;
mod models;
mod security;
mod ai;
mod rag;
mod pdf;
mod server;

use anyhow::Result;
use std::sync::Arc;

use config::Config;
use ai::OpenAIClient;
use rag::VectorStore;
use security::{RateLimiter, QueryValidator, CircuitBreaker};
use server::{AppState, start_server};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("🏍️  Bike Repair ChatBot - Starting...");

    // Load configuration
    let config = Config::from_env()?;
    config.validate()?;

    log::info!("✅ Configuration loaded");

    // Initialize OpenAI client
    let openai_client = Arc::new(OpenAIClient::new(
        config.openai_api_key.clone(),
        config.openai_chat_model.clone(),
        config.openai_embedding_model.clone(),
    ));
    log::info!("✅ OpenAI client initialized");

    // Initialize vector store (embedded Qdrant)
    let vector_store = Arc::new(
        VectorStore::new(&config.qdrant_path)
            .await
            .expect("Failed to initialize vector store"),
    );
    log::info!("✅ Vector store initialized ({})", config.qdrant_path);

    // Initialize security components
    let rate_limiter = Arc::new(RateLimiter::new(
        config.max_requests_per_minute,
        config.max_requests_per_hour,
    ));
    log::info!("✅ Rate limiter initialized");

    let query_validator = Arc::new(QueryValidator::new());
    log::info!("✅ Query validator initialized");

    let circuit_breaker = Arc::new(CircuitBreaker::new(
        config.circuit_breaker_threshold,
        config.circuit_breaker_timeout_seconds,
    ));
    log::info!("✅ Circuit breaker initialized");

    // Create application state
    let state = AppState {
        config: Arc::new(config),
        openai_client,
        rate_limiter: rate_limiter.clone(),
        query_validator,
        circuit_breaker,
    };

    log::info!("✅ Application state initialized");

    // Start periodic cleanup task for rate limiter
    let rate_limiter_cleanup = rate_limiter.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(600)); // 10 minutes
        loop {
            interval.tick().await;
            rate_limiter_cleanup.cleanup_old_entries();
        }
    });

    // Start HTTP server
    log::info!("🚀 Starting HTTP server...");
    start_server(state).await?;

    Ok(())
}
