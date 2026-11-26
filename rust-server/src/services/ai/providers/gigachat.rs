//! GigaChat AI provider implementation.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

use super::{AiCapability, AiError, AiProvider, AiQuestion, GenerationRequest, GenerationResponse};
use crate::services::ai::providers::ProviderConfig;

/// GigaChat API client.
pub struct GigaChatProvider {
    config: ProviderConfig,
    client: Client,
    oauth_token: Arc<Mutex<Option<String>>>,
    token_expires_at: Arc<Mutex<Option<i64>>>,
}

#[derive(Debug, Deserialize)]
struct GigaChatTokenResponse {
    access_token: String,
    expires_at: i64,
}

#[derive(Debug, Serialize)]
struct GigaChatRequest {
    model: String,
    messages: Vec<GigaChatMessage>,
    temperature: f64,
    max_tokens: usize,
}

#[derive(Debug, Serialize)]
struct GigaChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct GigaChatResponse {
    choices: Vec<GigaChatChoice>,
    usage: GigaChatUsage,
}

#[derive(Debug, Deserialize)]
struct GigaChatChoice {
    message: GigaChatMessageResponse,
}

#[derive(Debug, Deserialize)]
struct GigaChatMessageResponse {
    content: String,
}

#[derive(Debug, Deserialize)]
struct GigaChatUsage {
    total_tokens: u64,
}

impl GigaChatProvider {
    pub fn new(config: ProviderConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            oauth_token: Arc::new(Mutex::new(None)),
            token_expires_at: Arc::new(Mutex::new(None)),
        }
    }

    async fn get_oauth_token(&self) -> Result<String, AiError> {
        let mut token_lock = self.oauth_token.lock().await;
        let mut expires_lock = self.token_expires_at.lock().await;

        // Check if we have a valid token
        if let (Some(token), Some(expires_at)) = (token_lock.as_ref(), expires_lock.as_ref()) {
            let now = chrono::Utc::now().timestamp();
            if *expires_at > now + 60 {
                // Token valid for at least 1 minute
                return Ok(token.clone());
            }
        }

        // Need to refresh token
        info!("Refreshing GigaChat OAuth token");

        let credentials = self.config.api_key.as_ref().ok_or_else(|| {
            AiError::Unavailable("GigaChat credentials not configured".to_string())
        })?;

        let response = self
            .client
            .post("https://ngw.devices.sberbank.ru:9443/api/v2/oauth")
            .header("Authorization", format!("Basic {}", credentials))
            .header("RqUID", uuid::Uuid::new_v4().to_string())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[(
                "scope",
                &self
                    .config
                    .api_endpoint
                    .clone()
                    .unwrap_or_else(|| "GIGACHAT_API_PERS".to_string()),
            )])
            .send()
            .await
            .map_err(|e| AiError::Unavailable(format!("OAuth request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("GigaChat OAuth failed: {} - {}", status, body);
            return Err(AiError::Unavailable(format!(
                "OAuth failed with status {}",
                status
            )));
        }

        let token_response: GigaChatTokenResponse = response
            .json()
            .await
            .map_err(|e| AiError::InvalidResponse(format!("Failed to parse token: {}", e)))?;

        *token_lock = Some(token_response.access_token.clone());
        *expires_lock = Some(token_response.expires_at);

        info!("GigaChat OAuth token refreshed successfully");
        Ok(token_response.access_token)
    }

    fn build_prompt(&self, request: &GenerationRequest) -> String {
        let difficulty_map = match request.difficulty.as_str() {
            "easy" => "простые, для школьников",
            "hard" => "сложные, для экспертов",
            _ => "средней сложности",
        };

        format!(
            "Создай {} вопросов викторины по теме \"{}\" сложности {}.\n\
             Каждый вопрос должен иметь ровно {} варианта ответа.\n\
             Формат ответа (JSON):\n\
             {{\n\
               \"questions\": [\n\
                 {{\n\
                   \"question\": \"текст вопроса\",\n\
                   \"options\": [\"вариант1\", \"вариант2\", ...],\n\
                   \"correct_index\": 0,\n\
                   \"explanation\": \"объяснение правильного ответа\",\n\
                   \"hint\": \"подсказка\"\n\
                 }}\n\
               ]\n\
             }}\n\
             Верни ТОЛЬКО JSON без дополнительного текста.",
            request.count, request.topic, difficulty_map, request.num_options
        )
    }

    fn parse_questions(&self, content: &str) -> Result<Vec<AiQuestion>, AiError> {
        // Try to extract JSON from response
        let json_str = if content.contains("```json") {
            content
                .split("```json")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(content)
        } else if content.contains("```") {
            content
                .split("```")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(content)
        } else {
            content
        };

        #[derive(Debug, Deserialize)]
        struct QuestionsWrapper {
            questions: Vec<RawQuestion>,
        }

        #[derive(Debug, Deserialize)]
        struct RawQuestion {
            question: String,
            options: Vec<String>,
            correct_index: usize,
            explanation: Option<String>,
            hint: Option<String>,
        }

        let wrapper: QuestionsWrapper = serde_json::from_str(json_str).map_err(|e| {
            error!("Failed to parse GigaChat response: {}", e);
            AiError::InvalidResponse(format!("JSON parse error: {}", e))
        })?;

        Ok(wrapper
            .questions
            .into_iter()
            .map(|q| AiQuestion {
                question: q.question,
                options: q.options,
                correct_index: q.correct_index,
                explanation: q.explanation.unwrap_or_default(),
                hint: q.hint.unwrap_or_default(),
                difficulty: "medium".to_string(),
                category: None,
            })
            .collect())
    }
}

#[async_trait]
impl AiProvider for GigaChatProvider {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn capabilities(&self) -> &[AiCapability] {
        &self.config.capabilities
    }

    async fn generate_questions(
        &self,
        request: &GenerationRequest,
    ) -> Result<GenerationResponse, AiError> {
        let start = std::time::Instant::now();

        let token = self.get_oauth_token().await?;
        let prompt = self.build_prompt(request);

        let chat_request = GigaChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                GigaChatMessage {
                    role: "system".to_string(),
                    content: "Ты — генератор вопросов для викторины. Отвечай только валидным JSON."
                        .to_string(),
                },
                GigaChatMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            temperature: 0.7,
            max_tokens: 4000,
        };

        let response = self
            .client
            .post("https://gigachat.devices.sberbank.ru/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", token))
            .json(&chat_request)
            .send()
            .await
            .map_err(|e| {
                error!("GigaChat API request failed: {}", e);
                AiError::Unavailable(format!("API request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("GigaChat API error: {} - {}", status, body);
            return Err(AiError::InvalidResponse(format!("API error: {}", status)));
        }

        let chat_response: GigaChatResponse = response
            .json()
            .await
            .map_err(|e| AiError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        let content = chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| AiError::InvalidResponse("No choices in response".to_string()))?;

        let questions = self.parse_questions(&content)?;

        let latency_ms = start.elapsed().as_millis() as u64;
        info!(
            provider = "gigachat",
            questions = questions.len(),
            latency_ms = latency_ms,
            "Generated questions"
        );

        Ok(GenerationResponse {
            questions,
            provider_used: self.name().to_string(),
            tokens_used: chat_response.usage.total_tokens,
            latency_ms,
        })
    }

    async fn generate_hint(&self, question: &str) -> Result<String, AiError> {
        let token = self.get_oauth_token().await?;

        let chat_request = GigaChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                GigaChatMessage {
                    role: "system".to_string(),
                    content: "Ты даёшь короткие подсказки к вопросам викторины. Подсказка не должна раскрывать ответ.".to_string(),
                },
                GigaChatMessage {
                    role: "user".to_string(),
                    content: format!("Дай подсказку к вопросу: {}", question),
                },
            ],
            temperature: 0.5,
            max_tokens: 100,
        };

        let response = self
            .client
            .post("https://gigachat.devices.sberbank.ru/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", token))
            .json(&chat_request)
            .send()
            .await
            .map_err(|e| AiError::Unavailable(format!("API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AiError::InvalidResponse(format!(
                "API error: {}",
                response.status()
            )));
        }

        let chat_response: GigaChatResponse = response
            .json()
            .await
            .map_err(|e| AiError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| AiError::InvalidResponse("No choices in response".to_string()))
    }

    async fn generate_explanation(
        &self,
        question: &str,
        correct_answer: &str,
    ) -> Result<String, AiError> {
        let token = self.get_oauth_token().await?;

        let chat_request = GigaChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                GigaChatMessage {
                    role: "system".to_string(),
                    content: "Ты объясняешь правильные ответы к вопросам викторины. Объяснение должно быть кратким и информативным.".to_string(),
                },
                GigaChatMessage {
                    role: "user".to_string(),
                    content: format!(
                        "Вопрос: {}\nПравильный ответ: {}\nОбъясни, почему это правильный ответ.",
                        question, correct_answer
                    ),
                },
            ],
            temperature: 0.5,
            max_tokens: 200,
        };

        let response = self
            .client
            .post("https://gigachat.devices.sberbank.ru/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", token))
            .json(&chat_request)
            .send()
            .await
            .map_err(|e| AiError::Unavailable(format!("API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AiError::InvalidResponse(format!(
                "API error: {}",
                response.status()
            )));
        }

        let chat_response: GigaChatResponse = response
            .json()
            .await
            .map_err(|e| AiError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| AiError::InvalidResponse("No choices in response".to_string()))
    }
}
