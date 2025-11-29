use serde::{Deserialize, Serialize};

/// Chat request from client
#[derive(Debug, Clone, Deserialize)]
pub struct ChatRequest {
    /// User's query/message
    pub query: String,
    
    /// Optional session ID for conversation history
    #[serde(default)]
    pub session_id: Option<String>,
    
    /// Optional bike model filter for RAG retrieval
    #[serde(default)]
    pub bike_model: Option<String>,
}

/// Chat response to client
#[derive(Debug, Clone, Serialize)]
pub struct ChatResponse {
    /// AI-generated response
    pub response: String,
    
    /// Session ID for conversation tracking
    pub session_id: String,
    
    /// Sources/citations from manual
    #[serde(default)]
    pub sources: Vec<Source>,
    
    /// Rate limit information
    pub rate_limit_info: RateLimitInfo,
}

/// Source citation from manual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    /// Bike model this source is from
    pub bike_model: String,
    
    /// Page number in PDF
    pub page_number: Option<u32>,
    
    /// Section/chapter title
    pub section: Option<String>,
    
    /// Similarity score (0.0 to 1.0)
    pub relevance_score: f32,
}

/// Rate limit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    /// Requests remaining this minute
    pub remaining_minute: u32,
    
    /// Requests remaining this hour
    pub remaining_hour: u32,
    
    /// Seconds until limit resets
    pub reset_in_seconds: u64,
}

/// Single message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message role (user, assistant, system)
    pub role: String,
    
    /// Message content
    pub content: String,
    
    /// Timestamp
    #[serde(default)]
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

impl Message {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
            timestamp: Some(chrono::Utc::now()),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
            timestamp: Some(chrono::Utc::now()),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
            timestamp: None,
        }
    }
}

/// Error response
#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
    pub details: Option<String>,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            code: code.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}
