use warp::{reject::Rejection, Filter, Reply};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::server::handlers::*;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<crate::config::Config>,
    pub openai_client: Arc<crate::ai::OpenAIClient>,
    pub rate_limiter: Arc<crate::security::RateLimiter>,
    pub query_validator: Arc<crate::security::QueryValidator>,
    pub circuit_breaker: Arc<crate::security::CircuitBreaker>,
}

/// Create all routes
pub fn create_routes(
    state: AppState,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let state_filter = warp::any().map(move || state.clone());

    // Health check endpoint
    let health = warp::path("health")
        .and(warp::get())
        .and_then(handle_health);

    // Chat endpoint
    let chat = warp::path("chat")
        .and(warp::post())
        .and(warp::body::json())
        .and(state_filter.clone())
        .and(warp::addr::remote())
        .and_then(handle_chat);

    // Status endpoint (rate limit info)
    let status = warp::path("status")
        .and(warp::get())
        .and(state_filter.clone())
        .and(warp::addr::remote())
        .and_then(handle_status);

    // Combine routes under /api prefix
    let api = warp::path("api").and(health.or(chat).or(status));

    // Add CORS
    api.with(
        warp::cors()
            .allow_any_origin()
            .allow_methods(vec!["GET", "POST", "OPTIONS"])
            .allow_headers(vec!["Content-Type", "Authorization"])
    )
    .with(warp::log("api"))
}

/// Start the HTTP server
pub async fn start_server(state: AppState) -> anyhow::Result<()> {
    let host = state.config.server_host.parse::<std::net::IpAddr>()?;
    let port = state.config.server_port;
    let addr = SocketAddr::new(host, port);

    let routes = create_routes(state);

    log::info!("🚀 Server starting on http://{}", addr);
    log::info!("📍 Endpoints:");
    log::info!("   GET  /api/health  - Health check");
    log::info!("   POST /api/chat    - Chat with AI");
    log::info!("   GET  /api/status  - Rate limit status");

    warp::serve(routes).run(addr).await;

    Ok(())
}
