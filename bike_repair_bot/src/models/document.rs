use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// PDF document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Unique document ID
    pub id: String,
    
    /// Original filename
    pub filename: String,
    
    /// Detected bike model (Honda CBR600RR, Yamaha R1, etc.)
    pub bike_model: String,
    
    /// Manual year if detected
    pub year: Option<u32>,
    
    /// Manual type (repair, maintenance, parts, owner)
    pub manual_type: Option<String>,
    
    /// Upload timestamp
    pub uploaded_at: chrono::DateTime<chrono::Utc>,
    
    /// Number of pages
    pub page_count: u32,
    
    /// Number of chunks created
    pub chunk_count: usize,
    
    /// Processing status
    pub status: DocumentStatus,
}

impl Document {
    pub fn new(filename: impl Into<String>, bike_model: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            filename: filename.into(),
            bike_model: bike_model.into(),
            year: None,
            manual_type: None,
            uploaded_at: chrono::Utc::now(),
            page_count: 0,
            chunk_count: 0,
            status: DocumentStatus::Processing,
        }
    }
}

/// Document processing status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DocumentStatus {
    Processing,
    Completed,
    Failed,
}

/// Text chunk from a document with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    /// Unique chunk ID
    pub id: String,
    
    /// Parent document ID
    pub document_id: String,
    
    /// Chunk text content
    pub text: String,
    
    /// Chunk metadata
    pub metadata: ChunkMetadata,
    
    /// Embedding vector (generated later)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
}

impl DocumentChunk {
    pub fn new(
        document_id: impl Into<String>,
        text: impl Into<String>,
        metadata: ChunkMetadata,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            document_id: document_id.into(),
            text: text.into(),
            metadata,
            embedding: None,
        }
    }

    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }
}

/// Metadata attached to each chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    /// Bike model
    pub bike_model: String,
    
    /// Page number in PDF
    pub page_number: Option<u32>,
    
    /// Section/chapter heading
    pub section: Option<String>,
    
    /// Manual type
    pub manual_type: Option<String>,
    
    /// Year if applicable
    pub year: Option<u32>,
    
    /// Chunk index in document
    pub chunk_index: usize,
}

impl ChunkMetadata {
    pub fn new(bike_model: impl Into<String>) -> Self {
        Self {
            bike_model: bike_model.into(),
            page_number: None,
            section: None,
            manual_type: None,
            year: None,
            chunk_index: 0,
        }
    }
}

/// Upload response
#[derive(Debug, Clone, Serialize)]
pub struct UploadResponse {
    pub document_id: String,
    pub filename: String,
    pub status: String,
    pub message: String,
}
