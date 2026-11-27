# Retrieval-Augmented Generation (RAG)

## Overview

RAG (Retrieval-Augmented Generation) allows generating quiz questions based on specific content (text, documents, URLs) rather than relying solely on the AI model's internal knowledge. This enables:

- **Custom quiz generation** from specific content (articles, textbooks, documentation)
- **More accurate questions** grounded in provided context
- **Domain-specific content** support (proprietary materials, specialized topics)
- **Reduced hallucinations** by grounding questions in actual content

## Architecture

```
User provides context (text/PDF/URL)
         ↓
Text extraction and cleaning
         ↓
Context stored and chunked
         ↓
AI receives context + generation request
         ↓
Questions generated from context
```

### Components

1. **Context Source** - Input content (text, PDF, URL)
2. **Text Extractor** - Extracts and cleans text from various formats
3. **Context Manager** - Stores and manages context for rooms
4. **RAG Router** - Integrates context with AI question generation

## Usage

### 1. Upload Text Context

```bash
curl -X POST http://localhost:5000/api/rag/context/text \
  -H "Content-Type: application/json" \
  -d '{
    "text": "The capital of France is Paris. Paris is known for the Eiffel Tower, which was built in 1889.",
    "filename": "france_facts.txt"
  }'
```

Response:
```json
{
  "ok": true,
  "context_id": "ctx_abc123",
  "text_length": 105,
  "reading_time_secs": 30,
  "message": "Context uploaded: 105 characters, ~0 min reading time"
}
```

### 2. Upload URL Context

```bash
curl -X POST http://localhost:5000/api/rag/context/url \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://en.wikipedia.org/wiki/Paris"
  }'
```

Response:
```json
{
  "ok": true,
  "context_id": "ctx_def456",
  "text_length": 50000,
  "reading_time_secs": 1500,
  "message": "URL content fetched: 50000 characters, ~25 min reading time"
}
```

### 3. Upload Document (PDF/TXT)

```bash
curl -X POST http://localhost:5000/api/rag/context/document \
  -F "file=@textbook.pdf"
```

Response:
```json
{
  "ok": true,
  "context_id": "ctx_ghi789",
  "text_length": 100000,
  "reading_time_secs": 3000,
  "message": "Document uploaded: 100000 characters, ~50 min reading time"
}
```

### 4. Create Room with RAG Context

When creating a room, include the context:

```json
{
  "topic": "French Geography",
  "mode": "classic",
  "context_id": "ctx_abc123"
}
```

The AI will generate questions based on the provided context instead of general knowledge.

## API Reference

### Endpoints

#### `POST /api/rag/context/text`

Upload raw text as RAG context.

**Request:**
```json
{
  "text": "string (required, max 1MB)",
  "filename": "string (optional)"
}
```

**Response:**
```json
{
  "ok": true,
  "context_id": "string",
  "text_length": 123,
  "reading_time_secs": 60,
  "message": "string"
}
```

#### `POST /api/rag/context/url`

Fetch content from URL and use as RAG context.

**Request:**
```json
{
  "url": "string (required, must start with http:// or https://)"
}
```

**Response:** Same as text upload.

#### `POST /api/rag/context/document`

Upload document (PDF or TXT) as RAG context.

**Request:** `multipart/form-data`
- `file`: File upload (max 10MB, PDF or TXT only)

**Response:** Same as text upload.

#### `GET /api/rag/context/:id`

Retrieve context by ID.

**Response:**
```json
{
  "context_id": "string",
  "text": "string",
  "text_length": 123,
  "metadata": {
    "source_type": "text|url|pdf",
    "filename": "string",
    "created_at": 1234567890,
    "reading_time_secs": 60
  }
}
```

## Technical Details

### Context Processing

1. **Text Extraction**
   - HTML tags removed (for URL content)
   - Special characters preserved
   - Whitespace normalized

2. **Chunking**
   - Large contexts are chunked to ~8000 characters
   - Prevents token overflow in AI requests
   - Maintains coherence within chunks

3. **Reading Time Estimation**
   - Based on average reading speed (200 words/minute)
   - Helps users estimate quiz generation time

### Integration with AI Router

The AI router's `generate_questions_with_context()` method:

```rust
pub async fn generate_questions_with_context(
    &self,
    request: &GenerationRequest,
    context: &RagContext,
) -> Result<GenerationResponse, AiError> {
    // Create enhanced prompt with context
    let enhanced_request = GenerationRequest {
        topic: format!(
            "{}\n\n---\n\nBased on the following context, generate {} questions:\n\n{}\n\n---\n\nTopic: {}",
            request.topic,
            request.count,
            context.get_context_chunk(8000),
            request.topic
        ),
        // ... other fields
    };

    self.generate_questions(&enhanced_request).await
}
```

### Prompt Engineering

The enhanced prompt structure:

```
{original_topic}

---

Based on the following context, generate {count} questions:

{context_chunk}

---

Topic: {original_topic}
```

This ensures:
- AI focuses on provided context
- Questions are relevant to the content
- Original topic maintains focus

## Use Cases

### 1. Educational Content

Upload textbook chapters to generate quizzes for students:

```bash
# Upload chapter text
curl -X POST /api/rag/context/text \
  -d '{"text": "Chapter 3: Photosynthesis is the process..."}'

# Create quiz room
curl -X POST /api/rooms \
  -d '{"topic": "Photosynthesis", "context_id": "ctx_abc123"}'
```

### 2. Corporate Training

Generate quizzes from company documentation:

```bash
# Upload policy document
curl -X POST /api/rag/context/document \
  -F "file=@company_policy.pdf"

# Create compliance quiz
curl -X POST /api/rooms \
  -d '{"topic": "Company Policies", "context_id": "ctx_def456"}'
```

### 3. Research Papers

Create quizzes from academic papers:

```bash
# Upload paper URL
curl -X POST /api/rag/context/url \
  -d '{"url": "https://arxiv.org/abs/2301.12345"}'

# Create research quiz
curl -X POST /api/rooms \
  -d '{"topic": "Machine Learning Research", "context_id": "ctx_ghi789"}'
```

## Limitations

1. **Context Size**
   - Max 1MB for text uploads
   - Max 10MB for document uploads
   - Context chunked to ~8000 characters for AI

2. **Supported Formats**
   - Text: Plain text, HTML
   - Documents: PDF, TXT
   - URLs: HTML pages (not all formats supported)

3. **PDF Processing**
   - Current implementation is simplified
   - Complex PDFs (scanned images) may not extract text correctly
   - Future: Integration with `pdf-extract` crate

4. **URL Fetching**
   - 30-second timeout
   - Some sites may block automated requests
   - JavaScript-rendered content not supported

## Performance

### Benchmarks

| Operation | Time | Notes |
|-----------|------|-------|
| Text upload | <10ms | Instant processing |
| URL fetch | 1-5s | Depends on site speed |
| PDF extraction | 1-3s | Simplified implementation |
| Context chunking | <5ms | Fast string operations |

### Optimization

1. **Caching**
   - Contexts can be cached in Redis
   - Reduces repeated fetches for same URL

2. **Chunking Strategy**
   - Intelligent chunking preserves coherence
   - Overlapping chunks for better context

3. **Async Processing**
   - All operations are async
   - Non-blocking for other requests

## Security Considerations

1. **Input Validation**
   - URL format validation
   - File size limits
   - Content type verification

2. **Resource Limits**
   - Request timeouts (30s for URLs)
   - File size limits (10MB max)
   - Rate limiting on endpoints

3. **Data Privacy**
   - Context stored temporarily
   - Not persisted after room closes
   - No external storage of sensitive content

## Testing

Run RAG tests:

```bash
# Unit tests
cargo test --lib services::ai::rag

# Integration tests
cargo test --test rag_test
```

## Future Enhancements

1. **Advanced PDF Processing**
   - Integration with `pdf-extract` crate
   - OCR support for scanned documents
   - Table and image extraction

2. **Smarter Chunking**
   - Semantic chunking (by paragraphs, sections)
   - Overlapping chunks for better context
   - Priority-based chunking (important sections first)

3. **Vector Database Integration**
   - Store context embeddings
   - Semantic search for relevant sections
   - Better context retrieval

4. **Multi-Context Support**
   - Combine multiple contexts in one room
   - Weight contexts by importance
   - Context versioning

## Related Features

- [Dynamic Difficulty Adjustment](./dynamic_difficulty_adjustment.md) - Adjust difficulty based on performance
- [AI Question Generation](../../modules/ai-service.md) - Core question generation
- [Multi-Provider Support](../../modules/ai-service.md) - Use different AI providers

## References

- [RAG Paper](https://arxiv.org/abs/2005.11401) - Original RAG paper
- [LangChain RAG](https://python.langchain.com/docs/use_cases/question_answering/) - RAG implementation examples
- [LlamaIndex](https://docs.llamaindex.ai/) - RAG framework
