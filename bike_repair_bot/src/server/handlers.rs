use warp::{reject::Rejection, reply::Reply};
use std::net::SocketAddr;

use crate::models::{ChatRequest, ChatResponse, ErrorResponse, RateLimitInfo, Source};
use crate::server::routes::AppState;
use crate::ai::build_chat_prompt;

/// Health check handler
pub async fn handle_health() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&serde_json::json!({
        "status": "healthy",
        "service": "Bike Repair ChatBot",
        "version": env!("CARGO_PKG_VERSION"),
    })))
}

/// Chat handler
pub async fn handle_chat(
    req: ChatRequest,
    state: AppState,
    remote_addr: Option<SocketAddr>,
) -> Result<impl Reply, Rejection> {
    let ip = remote_addr
        .map(|addr| addr.ip())
        .unwrap_or_else(|| std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));

    log::info!("Chat request from {}: {}", ip, req.query);

    // 1. Check rate limit
    let rate_limit_info = match state.rate_limiter.check_and_record(ip) {
        Ok(info) => info,
        Err(e) => {
            log::warn!("Rate limit exceeded for {}: {}", ip, e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&ErrorResponse::new(
                    e.to_string(),
                    "RATE_LIMIT_EXCEEDED",
                )),
                warp::http::StatusCode::TOO_MANY_REQUESTS,
            ));
        }
    };

    // 2. Validate query (bike-related and safe)
    if let Err(e) = state.query_validator.validate(&req.query) {
        log::warn!("Invalid query from {}: {}", ip, e);
        return Ok(warp::reply::with_status(
            warp::reply::json(&ErrorResponse::new(e.to_string(), "INVALID_QUERY")),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // 3. Check circuit breaker
    if let Err(e) = state.circuit_breaker.check_request().await {
        log::error!("Circuit breaker open: {}", e);
        return Ok(warp::reply::with_status(
            warp::reply::json(&ErrorResponse::new(
                e.to_string(),
                "SERVICE_UNAVAILABLE",
            )),
            warp::http::StatusCode::SERVICE_UNAVAILABLE,
        ));
    }

    // 4. Build prompt (no RAG retrieval yet - that comes in Phase 5)
    let messages = build_chat_prompt(&req.query, None, &[]);

    // 5. Call OpenAI API
    let response_text = match state.openai_client.chat_completion(messages, Some(500)).await {
        Ok(text) => {
            state.circuit_breaker.record_success().await;
            text
        }
        Err(e) => {
            log::error!("OpenAI API error: {}", e);
            state.circuit_breaker.record_failure().await;
            return Ok(warp::reply::with_status(
                warp::reply::json(&ErrorResponse::new(
                    "Failed to generate response. Please try again.",
                    "AI_ERROR",
                )),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // 6. Build response
    let session_id = req.session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let response = ChatResponse {
        response: response_text,
        session_id,
        sources: Vec::new(), // Will populate when RAG is implemented
        rate_limit_info,
    };

    log::info!("Chat response sent to {}", ip);

    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        warp::http::StatusCode::OK,
    ))
}

/// Status handler - get rate limit info
pub async fn handle_status(
    state: AppState,
    remote_addr: Option<SocketAddr>,
) -> Result<impl Reply, Rejection> {
    let ip = remote_addr
        .map(|addr| addr.ip())
        .unwrap_or_else(|| std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));

    let rate_limit_info = state.rate_limiter.get_status(ip);

    Ok(warp::reply::json(&serde_json::json!({
        "rate_limit": rate_limit_info,
        "circuit_breaker": {
            "state": format!("{:?}", state.circuit_breaker.get_state().await),
        }
    })))
}
