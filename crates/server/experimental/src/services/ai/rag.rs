//! Retrieval-Augmented Generation (RAG) for context-aware question generation.
//!
//! RAG allows generating questions based on provided context (text, documents, URLs)
//! rather than relying solely on the AI model's internal knowledge. This enables:
//!
//! - Custom quiz generation from specific content
//! - More accurate and relevant questions
//! - Support for proprietary or domain-specific content
//! - Reduced hallucinations by grounding questions in provided text
//!
//! ## Architecture
//!
//! ```text
//! User provides context (text/PDF/URL)
//!         ↓
//! Text extraction and chunking
//!         ↓
//! Context stored in room
//!         ↓
//! AI receives context + generation request
//!         ↓
//! Questions generated from context
//! ```
//!
//! ## Usage
//!
//! ```rust
//! // Create room with context
//! let context = RagContext::from_text("The capital of France is Paris...");
//! let room = create_room_with_context(topic, Some(context)).await?;
//!
//! // AI will generate questions based on provided context
//! let questions = ai_router.generate_questions_with_context(&request, &context).await?;
//! ```

use std::time::Duration;

use anyhow::{Context as AnyhowContext, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Source type for RAG context.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextSource {
    /// Raw text provided directly
    Text(String),
    /// URL to fetch content from
    Url(String),
    /// Base64-encoded PDF document
    Pdf(String),
    /// Plain text document
    Document(String),
}

/// RAG context containing source text for question generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagContext {
    /// Original source of the context
    pub source: ContextSource,
    /// Extracted and cleaned text content
    pub text: String,
    /// Number of characters in the text
    pub text_length: usize,
    /// Optional metadata about the source
    pub metadata: Option<ContextMetadata>,
}

/// Metadata about the context source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    /// Source type (text, pdf, url)
    pub source_type: String,
    /// Original filename (if uploaded)
    pub filename: Option<String>,
    /// Creation timestamp
    pub created_at: i64,
    /// Estimated reading time in seconds
    pub reading_time_secs: u32,
}

impl RagContext {
    /// Create RAG context from raw text.
    pub fn from_text(text: String) -> Self {
        let text_length = text.len();
        let reading_time_secs = Self::estimate_reading_time(&text);

        Self {
            source: ContextSource::Text(text.clone()),
            text,
            text_length,
            metadata: Some(ContextMetadata {
                source_type: "text".to_string(),
                filename: None,
                created_at: chrono::Utc::now().timestamp(),
                reading_time_secs,
            }),
        }
    }

    /// Create RAG context from a URL (fetches content).
    pub async fn from_url(url: String) -> Result<Self> {
        info!(url = %url, "Fetching context from URL");

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        let response = client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch URL")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch URL: HTTP {}", response.status());
        }

        let text = response
            .text()
            .await
            .context("Failed to read response body")?;

        // Basic HTML cleanup (remove tags)
        let cleaned_text = Self::clean_html(&text);
        let text_length = cleaned_text.len();
        let reading_time_secs = Self::estimate_reading_time(&cleaned_text);

        Ok(Self {
            source: ContextSource::Url(url),
            text: cleaned_text,
            text_length,
            metadata: Some(ContextMetadata {
                source_type: "url".to_string(),
                filename: None,
                created_at: chrono::Utc::now().timestamp(),
                reading_time_secs,
            }),
        })
    }

    /// Create RAG context from base64-encoded PDF.
    pub fn from_pdf_base64(base64_data: String) -> Result<Self> {
        info!("Extracting text from PDF");

        // Decode base64
        let pdf_bytes = base64::decode(&base64_data)
            .context("Failed to decode base64 PDF")?;

        // Extract text from PDF (simplified - in production use pdf-extract crate)
        // For now, we'll just store a placeholder message
        let text = Self::extract_pdf_text(&pdf_bytes)?;
        let text_length = text.len();
        let reading_time_secs = Self::estimate_reading_time(&text);

        Ok(Self {
            source: ContextSource::Pdf(base64_data),
            text,
            text_length,
            metadata: Some(ContextMetadata {
                source_type: "pdf".to_string(),
                filename: None,
                created_at: chrono::Utc::now().timestamp(),
                reading_time_secs,
            }),
        })
    }

    /// Estimate reading time based on text length.
    /// Assumes average reading speed of 200 words per minute.
    fn estimate_reading_time(text: &str) -> u32 {
        let word_count = text.split_whitespace().count();
        let words_per_minute = 200;
        let minutes = (word_count as f32 / words_per_minute as f32).ceil() as u32;
        minutes * 60 // Convert to seconds
    }

    /// Clean HTML tags from text (basic implementation).
    fn clean_html(html: &str) -> String {
        // Remove HTML tags
        let re = regex::Regex::new(r"<[^>]+>").unwrap();
        let text = re.replace_all(html, " ");

        // Decode HTML entities
        let text = text
            .replace("&nbsp;", " ")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"");

        // Normalize whitespace
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// Extract text from PDF bytes (simplified implementation).
    fn extract_pdf_text(pdf_bytes: &[u8]) -> Result<String> {
        // In production, use pdf-extract or similar crate
        // For now, return a placeholder
        debug!(bytes = pdf_bytes.len(), "PDF text extraction");

        // Check if it's actually a PDF
        if pdf_bytes.len() < 4 || &pdf_bytes[0..4] != b"%PDF" {
            anyhow::bail!("Invalid PDF format");
        }

        // Placeholder: in production, implement proper PDF text extraction
        Ok("[PDF content - requires pdf-extract crate for full text extraction]".to_string())
    }

    /// Get a chunk of text for AI processing.
    /// Limits context size to prevent token overflow.
    pub fn get_context_chunk(&self, max_chars: usize) -> String {
        if self.text.len() <= max_chars {
            self.text.clone()
        } else {
            // Take first max_chars characters
            self.text[..max_chars].to_string()
        }
    }

    /// Get context summary for logging/debugging.
    pub fn summary(&self) -> String {
        format!(
            "RAG Context: {} chars, {} words, ~{} min reading time",
            self.text_length,
            self.text.split_whitespace().count(),
            self.metadata.as_ref().map(|m| m.reading_time_secs / 60).unwrap_or(0)
        )
    }
}

/// RAG-enhanced generation request.
#[derive(Debug, Clone)]
pub struct RagGenerationRequest {
    /// Base generation request
    pub base_request: super::GenerationRequest,
    /// RAG context to use for generation
    pub context: RagContext,
}

impl RagGenerationRequest {
    /// Create a new RAG generation request.
    pub fn new(base_request: super::GenerationRequest, context: RagContext) -> Self {
        Self {
            base_request,
            context,
        }
    }

    /// Get the context chunk for AI (limited to prevent token overflow).
    pub fn get_ai_context(&self) -> String {
        // Limit context to ~8000 characters (roughly 2000 tokens)
        self.context.get_context_chunk(8000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rag_context_from_text() {
        let text = "The capital of France is Paris. Paris is known for the Eiffel Tower.";
        let context = RagContext::from_text(text.to_string());

        assert_eq!(context.text, text);
        assert_eq!(context.text_length, text.len());
        assert!(context.metadata.is_some());
        assert_eq!(context.metadata.as_ref().unwrap().source_type, "text");
    }

    #[test]
    fn test_reading_time_estimation() {
        // 200 words should take ~1 minute
        let text = "word ".repeat(200);
        let time = RagContext::estimate_reading_time(&text);
        assert_eq!(time, 60); // 1 minute in seconds

        // 400 words should take ~2 minutes
        let text = "word ".repeat(400);
        let time = RagContext::estimate_reading_time(&text);
        assert_eq!(time, 120); // 2 minutes in seconds
    }

    #[test]
    fn test_html_cleaning() {
        let html = "<p>Hello <b>world</b></p>";
        let cleaned = RagContext::clean_html(html);
        assert_eq!(cleaned, "Hello world");
    }

    #[test]
    fn test_context_chunking() {
        let text = "A".repeat(10000);
        let context = RagContext::from_text(text);

        let chunk = context.get_context_chunk(5000);
        assert_eq!(chunk.len(), 5000);

        let full = context.get_context_chunk(20000);
        assert_eq!(full.len(), 10000);
    }

    #[test]
    fn test_context_summary() {
        let text = "word ".repeat(100);
        let context = RagContext::from_text(text);
        let summary = context.summary();

        assert!(summary.contains("RAG Context"));
        assert!(summary.contains("chars"));
        assert!(summary.contains("words"));
    }
}
