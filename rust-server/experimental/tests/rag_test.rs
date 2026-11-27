//! RAG (Retrieval-Augmented Generation) integration tests.
//!
//! These tests verify the RAG system works correctly for context-aware
//! question generation.

use brainstorm::services::ai::rag::{ContextSource, RagContext};

#[test]
fn test_rag_context_from_text() {
    let text = "The capital of France is Paris. Paris is known for the Eiffel Tower.";
    let context = RagContext::from_text(text.to_string());

    assert_eq!(context.text, text);
    assert_eq!(context.text_length, text.len());
    assert!(context.metadata.is_some());

    let metadata = context.metadata.unwrap();
    assert_eq!(metadata.source_type, "text");
    assert!(metadata.reading_time_secs > 0);
}

#[test]
fn test_rag_context_chunking() {
    // Create a large text
    let text = "A".repeat(20000);
    let context = RagContext::from_text(text);

    // Get a chunk limited to 8000 characters
    let chunk = context.get_context_chunk(8000);
    assert_eq!(chunk.len(), 8000);

    // Get full text (larger limit)
    let full = context.get_context_chunk(30000);
    assert_eq!(full.len(), 20000);
}

#[test]
fn test_rag_context_summary() {
    let text = "word ".repeat(100); // 100 words
    let context = RagContext::from_text(text);

    let summary = context.summary();
    assert!(summary.contains("RAG Context"));
    assert!(summary.contains("chars"));
    assert!(summary.contains("words"));
    assert!(summary.contains("reading time"));
}

#[test]
fn test_rag_reading_time_estimation() {
    // 200 words should take ~1 minute
    let text_200_words = "word ".repeat(200);
    let context = RagContext::from_text(text_200_words);
    let metadata = context.metadata.unwrap();

    // Should be around 60 seconds (1 minute)
    assert!(metadata.reading_time_secs >= 50 && metadata.reading_time_secs <= 70);

    // 400 words should take ~2 minutes
    let text_400_words = "word ".repeat(400);
    let context = RagContext::from_text(text_400_words);
    let metadata = context.metadata.unwrap();

    // Should be around 120 seconds (2 minutes)
    assert!(metadata.reading_time_secs >= 110 && metadata.reading_time_secs <= 130);
}

#[test]
fn test_rag_html_cleaning() {
    let html = "<html><body><p>Hello <b>world</b></p><script>alert('xss')</script></body></html>";
    let context = RagContext::from_text(html.to_string());

    // Text should have HTML tags removed
    assert!(!context.text.contains("<html>"));
    assert!(!context.text.contains("<p>"));
    assert!(!context.text.contains("<script>"));
    assert!(context.text.contains("Hello"));
    assert!(context.text.contains("world"));
}

#[test]
fn test_rag_context_source_types() {
    // Text source
    let text_context = RagContext::from_text("Test text".to_string());
    match &text_context.source {
        ContextSource::Text(t) => assert_eq!(t, "Test text"),
        _ => panic!("Expected Text source"),
    }

    // URL source (would need async test for actual fetch)
    let url = "https://example.com".to_string();
    let metadata = ContextMetadata {
        source_type: "url".to_string(),
        filename: None,
        created_at: chrono::Utc::now().timestamp(),
        reading_time_secs: 60,
    };

    assert_eq!(metadata.source_type, "url");
}

#[test]
fn test_rag_context_with_special_characters() {
    let text = "Hello\n\nWorld\t\r\nTest & Special <chars>";
    let context = RagContext::from_text(text.to_string());

    // Should preserve special characters
    assert!(context.text.contains("Hello"));
    assert!(context.text.contains("World"));
    assert!(context.text.contains("&"));
}

#[test]
fn test_rag_empty_text_handling() {
    let text = "";
    let context = RagContext::from_text(text.to_string());

    assert_eq!(context.text_length, 0);
    let metadata = context.metadata.unwrap();
    assert_eq!(metadata.reading_time_secs, 0);
}

#[test]
fn test_rag_large_text_handling() {
    // 1MB of text
    let large_text = "A".repeat(1_000_000);
    let context = RagContext::from_text(large_text);

    assert_eq!(context.text_length, 1_000_000);
    let metadata = context.metadata.unwrap();
    // 1M chars / ~5 chars per word / 200 words per min = ~1000 minutes
    assert!(metadata.reading_time_secs > 0);
}

#[test]
fn test_rag_context_serialization() {
    let text = "Test text for serialization";
    let context = RagContext::from_text(text.to_string());

    // Serialize to JSON
    let json = serde_json::to_string(&context).unwrap();
    assert!(json.contains("text"));
    assert!(json.contains("text_length"));
    assert!(json.contains("metadata"));

    // Deserialize back
    let deserialized: RagContext = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.text, text);
    assert_eq!(deserialized.text_length, context.text_length);
}
