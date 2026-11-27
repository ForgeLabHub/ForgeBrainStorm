//! RAG (Retrieval-Augmented Generation) routes.
//!
//! Provides endpoints for uploading context (text, documents, URLs)
//! and generating questions based on that context.

use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::services::ai::rag::{ContextMetadata, ContextSource, RagContext};
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/context/text", post(upload_text_context))
        .route("/context/url", post(upload_url_context))
        .route("/context/document", post(upload_document_context))
        .route("/context/:id", get(get_context))
}

/// Request to upload text context.
#[derive(Debug, Deserialize)]
struct UploadTextRequest {
    text: String,
    filename: Option<String>,
}

/// Response for context upload.
#[derive(Debug, Serialize)]
struct ContextUploadResponse {
    ok: bool,
    context_id: String,
    text_length: usize,
    reading_time_secs: u32,
    message: String,
}

/// Upload raw text as RAG context.
async fn upload_text_context(
    State(state): State<AppState>,
    Json(request): Json<UploadTextRequest>,
) -> AppResult<impl IntoResponse> {
    if request.text.trim().is_empty() {
        return Err(AppError::BadRequest("Text cannot be empty".to_string()));
    }

    if request.text.len() > 1_000_000 {
        return Err(AppError::BadRequest(
            "Text too large (max 1MB)".to_string(),
        ));
    }

    let context = RagContext::from_text(request.text);
    let context_id = generate_context_id();

    // Store context in room or session (simplified - in production use Redis)
    // For now, just return the context info
    let reading_time_secs = context
        .metadata
        .as_ref()
        .map(|m| m.reading_time_secs)
        .unwrap_or(0);

    Ok((
        StatusCode::CREATED,
        Json(ContextUploadResponse {
            ok: true,
            context_id,
            text_length: context.text_length,
            reading_time_secs,
            message: format!(
                "Context uploaded: {} characters, ~{} min reading time",
                context.text_length,
                reading_time_secs / 60
            ),
        }),
    ))
}

/// Request to upload URL context.
#[derive(Debug, Deserialize)]
struct UploadUrlRequest {
    url: String,
}

/// Upload content from URL as RAG context.
async fn upload_url_context(
    State(state): State<AppState>,
    Json(request): Json<UploadUrlRequest>,
) -> AppResult<impl IntoResponse> {
    if request.url.trim().is_empty() {
        return Err(AppError::BadRequest("URL cannot be empty".to_string()));
    }

    // Validate URL format
    if !request.url.starts_with("http://") && !request.url.starts_with("https://") {
        return Err(AppError::BadRequest(
            "Invalid URL format (must start with http:// or https://)".to_string(),
        ));
    }

    let context = RagContext::from_url(request.url.clone())
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to fetch URL: {}", e)))?;

    let context_id = generate_context_id();
    let reading_time_secs = context
        .metadata
        .as_ref()
        .map(|m| m.reading_time_secs)
        .unwrap_or(0);

    Ok((
        StatusCode::CREATED,
        Json(ContextUploadResponse {
            ok: true,
            context_id,
            text_length: context.text_length,
            reading_time_secs,
            message: format!(
                "URL content fetched: {} characters, ~{} min reading time",
                context.text_length,
                reading_time_secs / 60
            ),
        }),
    ))
}

/// Upload document (PDF, TXT) as RAG context.
async fn upload_document_context(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> AppResult<impl IntoResponse> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to read multipart: {}", e)))?
    {
        if field.name() == Some("file") {
            filename = field.file_name().map(|s| s.to_string());
            file_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("Failed to read file: {}", e)))?
                    .to_vec(),
            );
        }
    }

    let file_data =
        file_data.ok_or_else(|| AppError::BadRequest("No file uploaded".to_string()))?;

    if file_data.len() > 10_000_000 {
        return Err(AppError::BadRequest(
            "File too large (max 10MB)".to_string(),
        ));
    }

    // Check file type
    let filename_str = filename.as_deref().unwrap_or("unknown");
    let is_pdf = filename_str.to_lowercase().ends_with(".pdf");
    let is_txt = filename_str.to_lowercase().ends_with(".txt");

    let context = if is_pdf {
        // Encode PDF to base64
        let base64_data = base64::encode(&file_data);
        RagContext::from_pdf_base64(base64_data)
            .map_err(|e| AppError::BadRequest(format!("Failed to process PDF: {}", e)))?
    } else if is_txt {
        // Read as text
        let text = String::from_utf8(file_data)
            .map_err(|e| AppError::BadRequest(format!("Invalid UTF-8 in text file: {}", e)))?;
        RagContext::from_text(text)
    } else {
        return Err(AppError::BadRequest(
            "Unsupported file type (use PDF or TXT)".to_string(),
        ));
    };

    let context_id = generate_context_id();
    let reading_time_secs = context
        .metadata
        .as_ref()
        .map(|m| m.reading_time_secs)
        .unwrap_or(0);

    Ok((
        StatusCode::CREATED,
        Json(ContextUploadResponse {
            ok: true,
            context_id,
            text_length: context.text_length,
            reading_time_secs,
            message: format!(
                "Document uploaded: {} characters, ~{} min reading time",
                context.text_length,
                reading_time_secs / 60
            ),
        }),
    ))
}

/// Get context by ID.
async fn get_context(
    State(state): State<AppState>,
    axum::extract::Path(context_id): axum::extract::Path<String>,
) -> AppResult<impl IntoResponse> {
    // In production, fetch from Redis or database
    // For now, return placeholder
    Err(AppError::NotFound(
        "Context not found (storage not implemented)".to_string(),
    ))
}

/// Generate a unique context ID.
fn generate_context_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let id: u64 = rng.gen();
    format!("ctx_{:x}", id)
}
