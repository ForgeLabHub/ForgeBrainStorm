//! Mock AI provider for testing.
//!
//! Returns deterministic, valid questions without making real API calls.

use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;

use super::{
    AiCapability, AiError, AiProvider, AiQuestion, GenerationRequest, GenerationResponse,
};

/// Mock AI provider for testing.
pub struct MockProvider {
    call_count: AtomicUsize,
    capabilities: Vec<AiCapability>,
}

impl MockProvider {
    pub fn new() -> Self {
        Self {
            call_count: AtomicUsize::new(0),
            capabilities: vec![
                AiCapability::QuestionGeneration,
                AiCapability::HintGeneration,
                AiCapability::ExplanationGeneration,
            ],
        }
    }

    pub fn get_call_count(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }

    pub fn reset_call_count(&self) {
        self.call_count.store(0, Ordering::SeqCst);
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AiProvider for MockProvider {
    fn name(&self) -> &str {
        "mock"
    }

    async fn is_available(&self) -> bool {
        true
    }

    fn capabilities(&self) -> &[AiCapability] {
        &self.capabilities
    }

    async fn generate_questions(
        &self,
        request: &GenerationRequest,
    ) -> Result<GenerationResponse, AiError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);

        let mut questions = Vec::with_capacity(request.count);

        for i in 0..request.count {
            let mut options = Vec::with_capacity(request.num_options);

            // Generate deterministic options
            for j in 0..request.num_options {
                options.push(format!(
                    "Option {} for question {} about {}",
                    j + 1,
                    i + 1,
                    request.topic
                ));
            }

            // Deterministic correct index (cycling through options)
            let correct_index = i % request.num_options;

            questions.push(AiQuestion {
                question: format!(
                    "Question {} about {} (difficulty: {})",
                    i + 1,
                    request.topic,
                    request.difficulty
                ),
                options,
                correct_index,
                explanation: format!(
                    "Explanation for question {} about {}",
                    i + 1,
                    request.topic
                ),
                hint: format!("Hint for question {} about {}", i + 1, request.topic),
                difficulty: request.difficulty.clone(),
                category: Some(request.topic.clone()),
            });
        }

        Ok(GenerationResponse {
            questions,
            provider_used: "mock".to_string(),
            tokens_used: 0,
            latency_ms: 0,
        })
    }

    async fn generate_hint(&self, question: &str) -> Result<String, AiError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        Ok(format!("Hint for: {}", question))
    }

    async fn generate_explanation(
        &self,
        question: &str,
        correct_answer: &str,
    ) -> Result<String, AiError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        Ok(format!(
            "Explanation for '{}' with answer '{}'",
            question, correct_answer
        ))
    }
}

/// Mock AI router for testing that uses MockProvider.
/// This is a simplified version for unit tests that don't need the full AiRouter.
pub struct MockAiRouter {
    provider: MockProvider,
}

impl MockAiRouter {
    pub fn new() -> Self {
        Self {
            provider: MockProvider::new(),
        }
    }

    pub async fn generate_questions(
        &self,
        topic: &str,
        count: usize,
        difficulty: &str,
        num_options: usize,
    ) -> Result<GenerationResponse, AiError> {
        let request = GenerationRequest {
            topic: topic.to_string(),
            count,
            difficulty: difficulty.to_string(),
            num_options,
            capabilities: vec![AiCapability::QuestionGeneration],
            exclude_hashes: Vec::new(),
        };
        self.provider.generate_questions(&request).await
    }

    pub async fn generate_hint(&self, question: &str) -> Result<String, AiError> {
        self.provider.generate_hint(question).await
    }

    pub async fn generate_explanation(
        &self,
        question: &str,
        correct_answer: &str,
    ) -> Result<String, AiError> {
        self.provider
            .generate_explanation(question, correct_answer)
            .await
    }

    pub fn get_call_count(&self) -> usize {
        self.provider.get_call_count()
    }

    pub fn reset_call_count(&self) {
        self.provider.reset_call_count()
    }
}

impl Default for MockAiRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_generates_questions() {
        let provider = MockProvider::new();

        let request = GenerationRequest {
            topic: "Test Topic".to_string(),
            count: 3,
            difficulty: "medium".to_string(),
            num_options: 4,
            capabilities: vec![AiCapability::QuestionGeneration],
            exclude_hashes: Vec::new(),
        };

        let response = provider.generate_questions(&request).await.unwrap();

        assert_eq!(response.questions.len(), 3);
        assert_eq!(provider.get_call_count(), 1);
        assert_eq!(response.provider_used, "mock");

        for q in &response.questions {
            assert!(q.question.contains("Test Topic"));
            assert_eq!(q.options.len(), 4);
            assert!(q.correct_index < 4);
            assert!(q.explanation.contains("Test Topic"));
            assert!(q.hint.contains("Test Topic"));
        }
    }

    #[tokio::test]
    async fn test_mock_provider_generates_hint() {
        let provider = MockProvider::new();

        let hint = provider.generate_hint("What is 2+2?").await.unwrap();

        assert!(hint.contains("What is 2+2?"));
        assert_eq!(provider.get_call_count(), 1);
    }

    #[tokio::test]
    async fn test_mock_provider_generates_explanation() {
        let provider = MockProvider::new();

        let explanation = provider
            .generate_explanation("What is 2+2?", "4")
            .await
            .unwrap();

        assert!(explanation.contains("What is 2+2?"));
        assert!(explanation.contains("4"));
        assert_eq!(provider.get_call_count(), 1);
    }

    #[tokio::test]
    async fn test_mock_provider_tracks_calls() {
        let provider = MockProvider::new();

        assert_eq!(provider.get_call_count(), 0);

        let request = GenerationRequest {
            topic: "A".to_string(),
            count: 1,
            difficulty: "easy".to_string(),
            num_options: 2,
            capabilities: vec![AiCapability::QuestionGeneration],
            exclude_hashes: Vec::new(),
        };

        provider.generate_questions(&request).await.unwrap();
        assert_eq!(provider.get_call_count(), 1);

        provider.generate_questions(&request).await.unwrap();
        assert_eq!(provider.get_call_count(), 2);

        provider.generate_hint("test").await.unwrap();
        assert_eq!(provider.get_call_count(), 3);

        provider.reset_call_count();
        assert_eq!(provider.get_call_count(), 0);
    }

    #[tokio::test]
    async fn test_mock_provider_is_always_available() {
        let provider = MockProvider::new();
        assert!(provider.is_available().await);
    }

    #[tokio::test]
    async fn test_mock_provider_has_correct_capabilities() {
        let provider = MockProvider::new();
        let caps = provider.capabilities();
        assert!(caps.contains(&AiCapability::QuestionGeneration));
        assert!(caps.contains(&AiCapability::HintGeneration));
        assert!(caps.contains(&AiCapability::ExplanationGeneration));
    }

    #[tokio::test]
    async fn test_mock_ai_router() {
        let router = MockAiRouter::new();

        let response = router
            .generate_questions("Test", 2, "medium", 3)
            .await
            .unwrap();

        assert_eq!(response.questions.len(), 2);
        assert_eq!(router.get_call_count(), 1);

        router.reset_call_count();
        assert_eq!(router.get_call_count(), 0);
    }
}
