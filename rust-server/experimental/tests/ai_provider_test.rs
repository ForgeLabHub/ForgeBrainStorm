//! AI provider tests with mocks.
//!
//! These tests verify the AI client logic using the MockProvider,
//! without making real API calls.

use brainstorm::services::ai::mock::{MockAiRouter, MockProvider};
use brainstorm::services::ai::{
    AiCapability, AiProvider, GenerationRequest,
};

#[tokio::test]
async fn test_mock_provider_returns_questions() {
    let provider = MockProvider::new();

    let request = GenerationRequest {
        topic: "Test topic".to_string(),
        count: 5,
        difficulty: "medium".to_string(),
        num_options: 4,
        capabilities: vec![AiCapability::QuestionGeneration],
        exclude_hashes: Vec::new(),
    };

    let response = provider.generate_questions(&request).await.unwrap();

    assert_eq!(response.questions.len(), 5);
    assert!(response.questions.iter().all(|q| q.options.len() == 4));
    assert!(response.questions.iter().all(|q| q.correct_index < 4));
}

#[tokio::test]
async fn test_mock_provider_respects_question_count() {
    let provider = MockProvider::new();

    let request = GenerationRequest {
        topic: "Test".to_string(),
        count: 10,
        difficulty: "easy".to_string(),
        num_options: 4,
        capabilities: vec![AiCapability::QuestionGeneration],
        exclude_hashes: Vec::new(),
    };

    let response = provider.generate_questions(&request).await.unwrap();
    assert_eq!(response.questions.len(), 10);
}

#[tokio::test]
async fn test_mock_provider_respects_num_options() {
    let provider = MockProvider::new();

    let request = GenerationRequest {
        topic: "Test".to_string(),
        count: 3,
        difficulty: "medium".to_string(),
        num_options: 6,
        capabilities: vec![AiCapability::QuestionGeneration],
        exclude_hashes: Vec::new(),
    };

    let response = provider.generate_questions(&request).await.unwrap();
    assert!(response.questions.iter().all(|q| q.options.len() == 6));
}

#[tokio::test]
async fn test_mock_provider_hint() {
    let provider = MockProvider::new();

    let hint = provider.generate_hint("What is 2+2?").await.unwrap();

    assert!(!hint.is_empty());
    assert!(hint.contains("2+2"));
}

#[tokio::test]
async fn test_mock_provider_explanation() {
    let provider = MockProvider::new();

    let explanation = provider
        .generate_explanation("What is 2+2?", "4")
        .await
        .unwrap();

    assert!(!explanation.is_empty());
    assert!(explanation.contains("4"));
}

#[tokio::test]
async fn test_mock_provider_is_available() {
    let provider = MockProvider::new();
    assert!(provider.is_available().await);
}

#[tokio::test]
async fn test_mock_provider_name() {
    let provider = MockProvider::new();
    assert_eq!(provider.name(), "mock");
}

#[tokio::test]
async fn test_mock_provider_capabilities() {
    let provider = MockProvider::new();
    let caps = provider.capabilities();

    assert!(caps.contains(&AiCapability::QuestionGeneration));
    assert!(caps.contains(&AiCapability::HintGeneration));
    assert!(caps.contains(&AiCapability::ExplanationGeneration));
}

#[tokio::test]
async fn test_mock_ai_router_questions() {
    let router = MockAiRouter::new();

    let response = router
        .generate_questions("Test", 5, "medium", 4)
        .await
        .unwrap();

    assert_eq!(response.questions.len(), 5);
    assert_eq!(router.get_call_count(), 1);
}

#[tokio::test]
async fn test_mock_ai_router_hint() {
    let router = MockAiRouter::new();

    let hint = router.generate_hint("What is 2+2?").await.unwrap();

    assert!(!hint.is_empty());
    assert_eq!(router.get_call_count(), 1);
}

#[tokio::test]
async fn test_mock_ai_router_explanation() {
    let router = MockAiRouter::new();

    let explanation = router
        .generate_explanation("What is 2+2?", "4")
        .await
        .unwrap();

    assert!(!explanation.is_empty());
    assert_eq!(router.get_call_count(), 1);
}

#[tokio::test]
async fn test_mock_provider_call_counting() {
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

    provider.generate_explanation("q", "a").await.unwrap();
    assert_eq!(provider.get_call_count(), 4);
}

#[tokio::test]
async fn test_mock_provider_call_count_reset() {
    let provider = MockProvider::new();

    provider.generate_hint("test").await.unwrap();
    provider.generate_hint("test").await.unwrap();
    assert_eq!(provider.get_call_count(), 2);

    provider.reset_call_count();
    assert_eq!(provider.get_call_count(), 0);

    provider.generate_hint("test").await.unwrap();
    assert_eq!(provider.get_call_count(), 1);
}

#[tokio::test]
async fn test_mock_ai_router_call_counting() {
    let router = MockAiRouter::new();

    assert_eq!(router.get_call_count(), 0);

    router.generate_questions("A", 1, "easy", 2).await.unwrap();
    assert_eq!(router.get_call_count(), 1);

    router.generate_hint("test").await.unwrap();
    assert_eq!(router.get_call_count(), 2);

    router.reset_call_count();
    assert_eq!(router.get_call_count(), 0);
}

#[tokio::test]
async fn test_question_serialization() {
    use brainstorm::services::ai::AiQuestion;

    let question = AiQuestion {
        question: "What is 2+2?".to_string(),
        options: vec![
            "3".to_string(),
            "4".to_string(),
            "5".to_string(),
            "6".to_string(),
        ],
        correct_index: 1,
        explanation: "Basic arithmetic".to_string(),
        hint: "Think simple".to_string(),
        difficulty: "easy".to_string(),
        category: Some("Math".to_string()),
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
    assert_eq!(question.difficulty, deserialized.difficulty);
    assert_eq!(question.category, deserialized.category);
}

#[tokio::test]
async fn test_question_json_keys() {
    use brainstorm::services::ai::AiQuestion;

    let question = AiQuestion {
        question: "Test".to_string(),
        options: vec!["A".into(), "B".into()],
        correct_index: 0,
        explanation: "Exp".to_string(),
        hint: "Hint".to_string(),
        difficulty: "easy".to_string(),
        category: Some("Cat".to_string()),
    };

    let json = serde_json::to_value(&question).unwrap();

    // Verify correct_index is the key (not "correct")
    assert!(json.get("correct_index").is_some());
    assert_eq!(json["correct_index"], 0);
    assert!(json.get("question").is_some());
    assert!(json.get("options").is_some());
    assert!(json.get("explanation").is_some());
    assert!(json.get("hint").is_some());
    assert!(json.get("difficulty").is_some());
    assert!(json.get("category").is_some());
}

#[tokio::test]
async fn test_generation_request_serialization() {
    let request = GenerationRequest {
        topic: "History".to_string(),
        count: 5,
        difficulty: "hard".to_string(),
        num_options: 4,
        capabilities: vec![AiCapability::QuestionGeneration],
        exclude_hashes: vec!["abc123".to_string()],
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("History"));
    assert!(json.contains("hard"));
}
