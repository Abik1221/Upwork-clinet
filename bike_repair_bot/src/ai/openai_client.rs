use anyhow::Result;
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, ChatCompletionRequestAssistantMessageArgs,
        CreateChatCompletionRequestArgs, CreateEmbeddingRequestArgs, EmbeddingInput,
    },
    Client,
};

use crate::models::Message;

/// OpenAI API client wrapper
pub struct OpenAIClient {
    client: Client<OpenAIConfig>,
    chat_model: String,
    embedding_model: String,
}

impl OpenAIClient {
    pub fn new(api_key: impl Into<String>, chat_model: String, embedding_model: String) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(config);

        Self {
            client,
            chat_model,
            embedding_model,
        }
    }

    /// Generate a chat completion
    pub async fn chat_completion(
        &self,
        messages: Vec<Message>,
        max_tokens: Option<u16>,
    ) -> Result<String> {
        // Convert our Message type to OpenAI's message type
        let api_messages: Vec<ChatCompletionRequestMessage> = messages
            .into_iter()
            .map(|msg| match msg.role.as_str() {
                "system" => ChatCompletionRequestSystemMessageArgs::default()
                    .content(msg.content)
                    .build()
                    .unwrap()
                    .into(),
                "user" => ChatCompletionRequestUserMessageArgs::default()
                    .content(msg.content)
                    .build()
                    .unwrap()
                    .into(),
                "assistant" => ChatCompletionRequestAssistantMessageArgs::default()
                    .content(msg.content)
                    .build()
                    .unwrap()
                    .into(),
                _ => ChatCompletionRequestUserMessageArgs::default()
                    .content(msg.content)
                    .build()
                    .unwrap()
                    .into(),
            })
            .collect();

        // Build request
        let mut request = CreateChatCompletionRequestArgs::default();
        request.model(&self.chat_model).messages(api_messages);

        if let Some(tokens) = max_tokens {
            request.max_tokens(tokens);
        }

        let request = request.build()?;

        // Call API
        let response = self.client.chat().create(request).await?;

        // Extract response text
        let response_text = response
            .choices
            .first()
            .ok_or_else(|| anyhow::anyhow!("No response from OpenAI"))?
            .message
            .content
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Empty response from OpenAI"))?;

        log::debug!(
            "Chat completion: {} tokens used",
            response.usage.map(|u| u.total_tokens).unwrap_or(0)
        );

        Ok(response_text)
    }

    /// Generate embeddings for text
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.embedding_model)
            .input(EmbeddingInput::String(text.to_string()))
            .build()?;

        let response = self.client.embeddings().create(request).await?;

        let embedding = response
            .data
            .first()
            .ok_or_else(|| anyhow::anyhow!("No embedding returned"))?
            .embedding
            .clone();

        Ok(embedding)
    }

    /// Generate embeddings for multiple texts in batch
    pub async fn generate_embeddings_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // OpenAI supports up to 2048 inputs per request, but we'll use smaller batches
        const BATCH_SIZE: usize = 100;

        let mut all_embeddings = Vec::new();

        for chunk in texts.chunks(BATCH_SIZE) {
            let request = CreateEmbeddingRequestArgs::default()
                .model(&self.embedding_model)
                .input(EmbeddingInput::StringArray(chunk.to_vec()))
                .build()?;

            let response = self.client.embeddings().create(request).await?;

            let batch_embeddings: Vec<Vec<f32>> = response
                .data
                .into_iter()
                .map(|embedding_data| embedding_data.embedding)
                .collect();

            all_embeddings.extend(batch_embeddings);

            log::debug!("Generated {} embeddings in batch", chunk.len());
        }

        Ok(all_embeddings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a valid OpenAI API key
    // They are ignored by default to avoid API calls during normal testing

    #[tokio::test]
    #[ignore]
    async fn test_chat_completion() {
        let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
        let client = OpenAIClient::new(
            api_key,
            "gpt-4o-mini".to_string(),
            "text-embedding-3-small".to_string(),
        );

        let messages = vec![Message::user("What is 2+2?")];

        let response = client.chat_completion(messages, Some(100)).await.unwrap();
        assert!(!response.is_empty());
        println!("Response: {}", response);
    }

    #[tokio::test]
    #[ignore]
    async fn test_generate_embedding() {
        let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
        let client = OpenAIClient::new(
            api_key,
            "gpt-4o-mini".to_string(),
            "text-embedding-3-small".to_string(),
        );

        let embedding = client
            .generate_embedding("How to change motorcycle oil")
            .await
            .unwrap();

        assert_eq!(embedding.len(), 1536); // text-embedding-3-small dimension
    }
}
