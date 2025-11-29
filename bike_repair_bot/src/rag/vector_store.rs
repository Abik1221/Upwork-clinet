// Placeholder - will implement vector database later

use anyhow::Result;

pub struct VectorStore;

impl VectorStore {
    pub async fn new(_storage_path: &str) -> Result<Self> {
        Ok(Self)
    }
}
