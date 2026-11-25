//! Ollama local model provider implementation.
//!
//! Ollama runs AI models locally, no API key needed.
//! Supports any model that Ollama can run (llama2, mistral, etc.)

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use super::{
    AiCapability, AiError, AiProvider, AiQuestion, GenerationRequest, GenerationResponse,
    ProviderConfig,
};

/// Ollama API client.
pub struct OllamaProvider {
    config: ProviderConfig,
    client: Client,
}

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    format: String,
    options: OllamaOptions,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: f64,
    num_predict: usize,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
}

impl OllamaProvider {
    pub fn new(config: ProviderConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    fn endpoint(&self) -> &str {
        self.config
            .api_endpoint
            .as_deref()
            .unwrap_or("http://localhost:11434")
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
            error!("Failed to parse Ollama response: {}", e);
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
impl AiProvider for OllamaProvider {
    fn name(&self) -> &str {
        &self.config.name
    }

    async fn is_available(&self) -> bool {
        // Check if Ollama is running and model is available
        let url = format!("{}/api/tags", self.endpoint());
        match self.client.get(&url).send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    return false;
                }
                match response.json::<OllamaTagsResponse>().await {
                    Ok(tags) => tags.models.iter().any(|m| m.name == self.config.model),
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }

    fn capabilities(&self) -> &[AiCapability] {
        &self.config.capabilities
    }

    async fn generate_questions(
        &self,
        request: &GenerationRequest,
    ) -> Result<GenerationResponse, AiError> {
        let start = std::time::Instant::now();

        let prompt = self.build_prompt(request);
        let url = format!("{}/api/generate", self.endpoint());

        let ollama_request = OllamaRequest {
            model: self.config.model.clone(),
            prompt,
            stream: false,
            format: "json".to_string(),
            options: OllamaOptions {
                temperature: 0.7,
                num_predict: 4000,
            },
        };

        let response = self
            .client
            .post(&url)
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| {
                error!("Ollama API request failed: {}", e);
                AiError::Unavailable(format!("API request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Ollama API error: {} - {}", status, body);
            return Err(AiError::InvalidResponse(format!("API error: {}", status)));
        }

        let ollama_response: OllamaResponse = response.json().await.map_err(|e| {
            AiError::InvalidResponse(format!("Failed to parse response: {}", e))
        })?;

        let questions = self.parse_questions(&ollama_response.response)?;

        let latency_ms = start.elapsed().as_millis() as u64;
        info!(
            provider = "ollama",
            model = %self.config.model,
            questions = questions.len(),
            latency_ms = latency_ms,
            "Generated questions with local model"
        );

        Ok(GenerationResponse {
            questions,
            provider_used: self.name().to_string(),
            tokens_used: 0, // Ollama doesn't always report token usage
            latency_ms,
        })
    }

    async fn generate_hint(&self, question: &str) -> Result<String, AiError> {
        let url = format!("{}/api/generate", self.endpoint());

        let ollama_request = OllamaRequest {
            model: self.config.model.clone(),
            prompt: format!(
                "Дай короткую подсказку к вопросу викторины (не раскрывай ответ): {}",
                question
            ),
            stream: false,
            format: String::new(),
            options: OllamaOptions {
                temperature: 0.5,
                num_predict: 100,
            },
        };

        let response = self
            .client
            .post(&url)
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| AiError::Unavailable(format!("API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AiError::InvalidResponse(format!(
                "API error: {}",
                response.status()
            )));
        }

        let ollama_response: OllamaResponse = response.json().await.map_err(|e| {
            AiError::InvalidResponse(format!("Failed to parse response: {}", e))
        })?;

        Ok(ollama_response.response)
    }

    async fn generate_explanation(
        &self,
        question: &str,
        correct_answer: &str,
    ) -> Result<String, AiError> {
        let url = format!("{}/api/generate", self.endpoint());

        let ollama_request = OllamaRequest {
            model: self.config.model.clone(),
            prompt: format!(
                "Вопрос: {}\nПравильный ответ: {}\nОбъясни, почему это правильный ответ.",
                question, correct_answer
            ),
            stream: false,
            format: String::new(),
            options: OllamaOptions {
                temperature: 0.5,
                num_predict: 200,
            },
        };

        let response = self
            .client
            .post(&url)
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| AiError::Unavailable(format!("API request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AiError::InvalidResponse(format!(
                "API error: {}",
                response.status()
            )));
        }

        let ollama_response: OllamaResponse = response.json().await.map_err(|e| {
            AiError::InvalidResponse(format!("Failed to parse response: {}", e))
        })?;

        Ok(ollama_response.response)
    }
}
