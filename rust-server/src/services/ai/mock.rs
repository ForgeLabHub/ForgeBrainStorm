//! Mock AI provider for testing.
//!
//! Returns deterministic, valid questions without making real API calls.

use std::sync::atomic::{AtomicUsize, Ordering};
use async_trait::async_trait;
use serde_json::json;
use super::{AiProvider, AiQuestion, AiRouter};
use crate::error::{AppError, AppResult};

/// Mock AI provider for testing.
pub struct MockProvider {
    call_count: AtomicUsize,
}

impl MockProvider {
    pub fn new() -> Self {
        Self {
            call_count: AtomicUsize::new(0),
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
    async fn is_healthy(&self) -> bool {
        true
    }

    async fn generate_questions(
        &self,
        topic: &str,
        count: usize,
        difficulty: &str,
        num_options: usize,
    ) -> AppResult<Vec<AiQuestion>> {
        self.call_count.fetch_add(1, Ordering::SeqCst);

        let mut questions = Vec::with_capacity(count);

        for i in 0..count {
            let mut options = Vec::with_capacity(num_options);
            
            // Generate deterministic options
            for j in 0..num_options {
                options.push(format!("Option {} for question {} about {}", j + 1, i + 1, topic));
            }

            // Deterministic correct index (cycling through options)
            let correct_index = i % num_options;

            questions.push(AiQuestion {
                question: format!("Question {} about {} (difficulty: {})", i + 1, topic, difficulty),
                options,
                correct_index,
                explanation: format!("Explanation for question {} about {}", i + 1, topic),
                hint: format!("Hint for question {} about {}", i + 1, topic),
                category: Some(topic.to_string()),
            });
        }

        Ok(questions)
    }

    async fn generate_hint(&self, question: &str) -> AppResult<String> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        Ok(format!("Hint for: {}", question))
    }

    async fn generate_explanation(&self, question: &str, correct_answer: &str) -> AppResult<String> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        Ok(format!("Explanation for '{}' with answer '{}'", question, correct_answer))
    }

    fn name(&self) -> &str {
        "mock"
    }
}

/// Mock AI router for testing that uses MockProvider.
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
    ) -> AppResult<Vec<AiQuestion>> {
        self.provider
            .generate_questions(topic, count, difficulty, num_options)
            .await
    }

    pub async fn generate_hint(&self, question: &str) -> AppResult<String> {
        self.provider.generate_hint(question).await
    }

    pub async fn generate_explanation(&self, question: &str, correct_answer: &str) -> AppResult<String> {
        self.provider.generate_explanation(question, correct_answer).await
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
        
        let questions = provider
            .generate_questions("Test Topic", 3, "medium", 4)
            .await
            .unwrap();

        assert_eq!(questions.len(), 3);
        assert_eq!(provider.get_call_count(), 1);

        for (i, q) in questions.iter().enumerate() {
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

        provider.generate_questions("A", 1, "easy", 2).await.unwrap();
        assert_eq!(provider.get_call_count(), 1);

        provider.generate_questions("B", 1, "easy", 2).await.unwrap();
        assert_eq!(provider.get_call_count(), 2);

        provider.generate_hint("test").await.unwrap();
        assert_eq!(provider.get_call_count(), 3);

        provider.reset_call_count();
        assert_eq!(provider.get_call_count(), 0);
    }

    #[tokio::test]
    async fn test_mock_ai_router() {
        let router = MockAiRouter::new();
        
        let questions = router
            .generate_questions("Test", 2, "medium", 3)
            .await
            .unwrap();

        assert_eq!(questions.len(), 2);
        assert_eq!(router.get_call_count(), 1);

        router.reset_call_count();
        assert_eq!(router.get_call_count(), 0);
    }
}
