//! AI provider implementations.

pub mod gigachat;
pub mod ollama;
pub mod openai;

use async_trait::async_trait;

pub use super::{AiCapability, AiQuestion, ProviderConfig};
use super::{AiError, AiProvider, GenerationRequest, GenerationResponse};

/// Re-export provider structs.
pub use gigachat::GigaChatProvider;
pub use ollama::OllamaProvider;
pub use openai::OpenAIProvider;

/// Provider wrapper for dynamic dispatch.
pub struct ProviderBox {
    inner: Box<dyn AiProvider>,
}

impl ProviderBox {
    pub fn new<P: AiProvider + 'static>(provider: P) -> Self {
        Self {
            inner: Box::new(provider),
        }
    }
}

#[async_trait]
impl AiProvider for ProviderBox {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn capabilities(&self) -> &[super::AiCapability] {
        self.inner.capabilities()
    }

    async fn generate_questions(
        &self,
        request: &GenerationRequest,
    ) -> Result<GenerationResponse, AiError> {
        self.inner.generate_questions(request).await
    }

    async fn generate_hint(&self, question: &str) -> Result<String, AiError> {
        self.inner.generate_hint(question).await
    }

    async fn generate_explanation(
        &self,
        question: &str,
        correct_answer: &str,
    ) -> Result<String, AiError> {
        self.inner
            .generate_explanation(question, correct_answer)
            .await
    }
}
