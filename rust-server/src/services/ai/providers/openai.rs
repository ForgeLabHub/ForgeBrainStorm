//! OpenAI API provider implementation.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use super::{
    AiCapability, AiError, AiProvider, AiQuestion, GenerationRequest, GenerationResponse,
    ProviderConfig,
};

/// OpenAI API client.
pub struct OpenAIProvider {
    config: ProviderConfig,
    client: Client,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f64,
    max_tokens: usize,
    response_format: ResponseFormat,
}

#[derive(Debug, Serialize)]
struct ResponseFormat {
    r#type: String,
}

#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessageResponse,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessageResponse {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    total_tokens: u64,
}

impl OpenAIProvider {
    pub fn new(config: ProviderConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    fn build_prompt(&self, request: &GenerationRequest) -> String {
        let difficulty_map = match request.difficulty.as_str() {
            "easy" => "simple, for school students",
            "hard" => "difficult, for experts",
            _ => "medium difficulty",
        };

        format!(
            "Create {} quiz questions about \"{}\" at {} level.\n\
             Each question must have exactly {} answer options.\n\
             Return JSON format:\n\
             {{\n\
               \"questions\": [\n\
                 {{\n\
                   \"question\": \"question text\",\n\
                   \"options\": [\"option1\", \"option2\", ...],\n\
                   \"correct_index\": 0,\n\
                   \"explanation\": \"explanation of correct answer\",\n\
                   \"hint\": \"hint\"\n\
                 }}\n\
               ]\n\
             }}\n\
             Return ONLY JSON without additional text.",
            request.count, request.topic, difficulty_map, request.num_options
        )
    }

    fn parse_questions(&self, content: &str) -> Result<Vec<AiQuestion>, AiError> {
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

        let wrapper: QuestionsWrapper = serde_json::from_str(content).map_err(|e| {
            error!("Failed to parse OpenAI response: {}", e);
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
impl AiProvider for OpenAIProvider {
    fn name(&self) -> &str {
        &self.config.name
    }

    async fn is_available(&self) -> bool {
        self.config.api_key.is_some()
    }

    fn capabilities(&self) -> &[AiCapability] {
        &self.config.capabilities
    }

    async fn generate_questions(
        &self,
        request: &GenerationRequest,
    ) -> Result<GenerationResponse, AiError> {
        let start = std::time::Instant::now();

        let api_key = self.config.api_key.as_ref().ok_or_else(|| {
            AiError::Unavailable("OpenAI API key not configured".to_string())
        })?;

        let prompt = self.build_prompt(request);
        let endpoint = self
            .config
            .api_endpoint
            .as_deref()
            .unwrap_or("https://api.openai.com/v1/chat/completions");

        let chat_request = OpenAIRequest {
            model: self.config.model.clone(),
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: "You are a quiz question generator. Respond only with valid JSON.".to_string(),
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            temperature: 0.7,
            max_tokens: 4000,
            response_format: ResponseFormat {
                r#type: "json_object".to_string(),
            },
        };

        let response = self
            .client
            .post(endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&chat_request)
            .send()
            .await
            .map_err(|e| {
                error!("OpenAI API request failed: {}", e);
                AiError::Unavailable(format!("API request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("OpenAI API error: {} - {}", status, body);
            return Err(AiError::InvalidResponse(format!("API error: {}", status)));
        }

        let chat_response: OpenAIResponse = response.json().await.map_err(|e| {
            AiError::InvalidResponse(format!("Failed to parse response: {}", e))
        })?;

        let content = chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| AiError::InvalidResponse("No choices in response".to_string()))?;

        let questions = self.parse_questions(&content)?;

        let latency_ms = start.elapsed().as_millis() as u64;
        info!(
            provider = "openai",
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
        let api_key = self.config.api_key.as_ref().ok_or_else(|| {
            AiError::Unavailable("OpenAI API key not configured".to_string())
        })?;

        let endpoint = self
            .config
            .api_endpoint
            .as_deref()
            .unwrap_or("https://api.openai.com/v1/chat/completions");

        let chat_request = OpenAIRequest {
            model: self.config.model.clone(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: format!(
                    "Give a short hint for this quiz question (don't reveal the answer): {}",
                    question
                ),
            }],
            temperature: 0.5,
            max_tokens: 100,
            response_format: ResponseFormat {
                r#type: "text".to_string(),
            },
        };

        let response = self
            .client
            .post(endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
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

        let chat_response: OpenAIResponse = response.json().await.map_err(|e| {
            AiError::InvalidResponse(format!("Failed to parse response: {}", e))
        })?;

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
        let api_key = self.config.api_key.as_ref().ok_or_else(|| {
            AiError::Unavailable("OpenAI API key not configured".to_string())
        })?;

        let endpoint = self
            .config
            .api_endpoint
            .as_deref()
            .unwrap_or("https://api.openai.com/v1/chat/completions");

        let chat_request = OpenAIRequest {
            model: self.config.model.clone(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: format!(
                    "Question: {}\nCorrect answer: {}\nExplain why this is the correct answer.",
                    question, correct_answer
                ),
            }],
            temperature: 0.5,
            max_tokens: 200,
            response_format: ResponseFormat {
                r#type: "text".to_string(),
            },
        };

        let response = self
            .client
            .post(endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
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

        let chat_response: OpenAIResponse = response.json().await.map_err(|e| {
            AiError::InvalidResponse(format!("Failed to parse response: {}", e))
        })?;

        chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| AiError::InvalidResponse("No choices in response".to_string()))
    }
}
