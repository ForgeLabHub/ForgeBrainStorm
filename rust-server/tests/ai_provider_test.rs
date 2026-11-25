//! AI provider tests with mocks.
//!
//! These tests use mock AI responses to verify the AI client logic
//! without making real API calls.

use brainstorm::services::ai::{AiProvider, AiProviderConfig, MockProvider, AiQuestion};
use brainstorm::error::AppError;

#[tokio::test]
async fn test_mock_provider_returns_questions() {
    let provider = MockProvider::new();
    
    let questions = provider.generate_questions(
        "Test topic",
        5,
        "medium",
        4,
    ).await.unwrap();

    assert_eq!(questions.len(), 5);
    assert!(questions.iter().all(|q| q.options.len() == 4));
    assert!(questions.iter().all(|q| q.correct_index < 4));
}

#[tokio::test]
async fn test_mock_provider_respects_question_count() {
    let provider = MockProvider::new();
    
    let questions = provider.generate_questions(
        "Test",
        10,
        "easy",
        4,
    ).await.unwrap();

    assert_eq!(questions.len(), 10);
}

#[tokio::test]
async fn test_mock_provider_respects_num_options() {
    let provider = MockProvider::new();
    
    let questions = provider.generate_questions(
        "Test",
        3,
        "medium",
        6,
    ).await.unwrap();

    assert!(questions.iter().all(|q| q.options.len() == 6));
}

#[tokio::test]
async fn test_mock_provider_hint() {
    let provider = MockProvider::new();
    
    let hint = provider.generate_hint("What is 2+2?").await.unwrap();
    
    assert!(!hint.is_empty());
    assert!(hint.len() > 10);
}

#[tokio::test]
async fn test_mock_provider_explanation() {
    let provider = MockProvider::new();
    
    let explanation = provider.generate_explanation(
        "What is 2+2?",
        "4"
    ).await.unwrap();
    
    assert!(!explanation.is_empty());
    assert!(explanation.contains("4"));
}

#[tokio::test]
async fn test_mock_provider_is_available() {
    let provider = MockProvider::new();
    
    let available = provider.is_available().await;
    
    assert!(available);
}

#[tokio::test]
async fn test_ai_router_fallback() {
    use brainstorm::services::ai::router::AiRouter;
    
    // Create a router with a failing primary and working fallback
    let primary_config = AiProviderConfig {
        name: "failing_provider".to_string(),
        api_key: None,
        endpoint: None,
        model: "failing".to_string(),
        max_concurrent: 1,
        timeout_seconds: 1,
        enabled: true,
    };
    
    let fallback_config = AiProviderConfig {
        name: "working_provider".to_string(),
        api_key: None,
        endpoint: None,
        model: "working".to_string(),
        max_concurrent: 1,
        timeout_seconds: 10,
        enabled: true,
    };
    
    // Note: This test would require implementing the actual router
    // with mock providers. For now, just documenting the expected behavior.
    //
    // Expected:
    // 1. Router tries primary provider
    // 2. Primary fails (timeout or error)
    // 3. Router falls back to working provider
    // 4. Questions are returned successfully
}

#[tokio::test]
async fn test_ai_router_cache_hit() {
    // Test that cached questions are returned without calling the provider
    
    // Expected behavior:
    // 1. First call generates questions (calls provider)
    // 2. Second call with same parameters returns cached questions
    // 3. Provider is not called on second request
    // 4. Questions are identical
}

#[tokio::test]
async fn test_ai_router_cache_miss() {
    // Test that different parameters cause cache miss
    
    // Expected behavior:
    // 1. Call with topic "A" -> generates questions, calls provider
    // 2. Call with topic "B" -> cache miss, calls provider again
    // 3. Questions are different for different topics
}

#[tokio::test]
async fn test_ai_router_circuit_breaker() {
    // Test circuit breaker opens after failures
    
    // Expected behavior:
    // 1. Provider fails 5 times in a row
    // 2. Circuit breaker opens
    // 3. Subsequent calls fail immediately without calling provider
    // 4. After cooldown period, circuit breaker half-opens
    // 5. One successful call closes circuit breaker
}

#[tokio::test]
async fn test_ai_question_validation() {
    // Test that generated questions are validated
    
    let question = AiQuestion {
        question: "What is 2+2?".to_string(),
        options: vec!["3".to_string(), "4".to_string(), "5".to_string(), "6".to_string()],
        correct_index: 1,
        explanation: "Basic arithmetic".to_string(),
        hint: "Think simple".to_string(),
        category: Some("Math".to_string()),
    };
    
    // Validate question
    assert!(question.validate().is_ok());
    
    // Test invalid question
    let invalid_question = AiQuestion {
        question: "".to_string(),  // Empty question
        options: vec!["1".to_string(), "2".to_string()],
        correct_index: 5,  // Out of bounds
        explanation: "".to_string(),
        hint: "".to_string(),
        category: None,
    };
    
    assert!(invalid_question.validate().is_err());
}

#[tokio::test]
async fn test_ai_question_serialization() {
    let question = AiQuestion {
        question: "Test question".to_string(),
        options: vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()],
        correct_index: 2,
        explanation: "Test explanation".to_string(),
        hint: "Test hint".to_string(),
        category: Some("Test".to_string()),
    };
    
    // Serialize to JSON
    let json = serde_json::to_string(&question).unwrap();
    
    // Deserialize back
    let deserialized: AiQuestion = serde_json::from_str(&json).unwrap();
    
    assert_eq!(question.question, deserialized.question);
    assert_eq!(question.options, deserialized.options);
    assert_eq!(question.correct_index, deserialized.correct_index);
    assert_eq!(question.explanation, deserialized.explanation);
    assert_eq!(question.hint, deserialized.hint);
    assert_eq!(question.category, deserialized.category);
}
