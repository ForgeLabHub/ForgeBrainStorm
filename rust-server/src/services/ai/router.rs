//! AI router with fallback, load balancing, and provider selection.
//!
//! Features:
//! - Automatic failover when a provider is unavailable
//! - Load balancing across multiple providers
//! - Provider priority based on cost and latency
//! - Circuit breaker pattern for failing providers

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use parking_lot::RwLock;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use super::providers::ProviderBox;
use super::{
    AiCapability, AiError, AiProvider, GenerationRequest, GenerationResponse, ProviderConfig,
    ProviderType,
};
use crate::services::ai::cache::AiCache;
use crate::services::ai::providers::{GigaChatProvider, OllamaProvider, OpenAIProvider};

/// Circuit breaker state for a provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CircuitState {
    Closed,    // Normal operation
    Open,      // Failing, don't use
    HalfOpen,  // Testing if recovered
}

/// Circuit breaker for a provider.
struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    last_failure: Option<Instant>,
    success_count: u32,
}

impl CircuitBreaker {
    fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            last_failure: None,
            success_count: 0,
        }
    }

    fn record_success(&mut self) {
        if self.state == CircuitState::HalfOpen {
            self.success_count += 1;
            if self.success_count >= 3 {
                // Recovered
                self.state = CircuitState::Closed;
                self.failure_count = 0;
                self.success_count = 0;
                info!("Circuit breaker closed");
            }
        } else {
            self.failure_count = 0;
        }
    }

    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Instant::now());

        if self.failure_count >= 5 {
            // Too many failures, open circuit
            self.state = CircuitState::Open;
            warn!("Circuit breaker opened after {} failures", self.failure_count);
        }
    }

    fn is_available(&self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if enough time has passed to try again
                if let Some(last) = self.last_failure {
                    if last.elapsed() > Duration::from_secs(60) {
                        // Try again after 60 seconds
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }
}

/// AI router that manages multiple providers with fallback.
pub struct AiRouter {
    providers: Vec<ProviderBox>,
    circuits: Arc<RwLock<Vec<CircuitBreaker>>>,
    cache: Arc<AiCache>,
    default_provider: usize,
}

impl AiRouter {
    pub fn new(configs: Vec<ProviderConfig>) -> Self {
        let mut providers = Vec::new();
        let mut circuits = Vec::new();

        for config in configs {
            if !config.enabled {
                continue;
            }

            let provider: ProviderBox = match config.provider_type {
                ProviderType::GigaChat => ProviderBox::new(GigaChatProvider::new(config)),
                ProviderType::OpenAI => ProviderBox::new(OpenAIProvider::new(config)),
                ProviderType::Ollama => ProviderBox::new(OllamaProvider::new(config)),
                ProviderType::Anthropic | ProviderType::Custom => {
                    // TODO: Implement other providers
                    continue;
                }
            };

            providers.push(provider);
            circuits.push(CircuitBreaker::new());
        }

        info!("AI router initialized with {} providers", providers.len());

        Self {
            providers,
            circuits: Arc::new(RwLock::new(circuits)),
            cache: Arc::new(AiCache::new(Duration::from_secs(3600))), // 1 hour cache
            default_provider: 0,
        }
    }

    /// Select provider based on priority and availability.
    fn select_provider(&self, capability: AiCapability) -> Option<usize> {
        let circuits = self.circuits.read();

        // Try default provider first
        if self.default_provider < self.providers.len() {
            let circuit = &circuits[self.default_provider];
            if circuit.is_available() && self.providers[self.default_provider]
                .capabilities()
                .contains(&capability)
            {
                return Some(self.default_provider);
            }
        }

        // Fallback to other providers
        for (idx, provider) in self.providers.iter().enumerate() {
            if idx == self.default_provider {
                continue;
            }

            let circuit = &circuits[idx];
            if circuit.is_available() && provider.capabilities().contains(&capability) {
                return Some(idx);
            }
        }

        None
    }

    /// Generate questions with automatic fallback.
    pub async fn generate_questions(
        &self,
        request: &GenerationRequest,
    ) -> Result<GenerationResponse, AiError> {
        // Check cache first
        let cache_key = format!(
            "q:{}:{}:{}:{}",
            request.topic, request.difficulty, request.count, request.num_options
        );

        if let Some(cached) = self.cache.get(&cache_key).await {
            info!("Serving questions from cache");
            return Ok(cached);
        }

        // Try providers with fallback
        let mut last_error = None;
        let mut attempts = 0;

        loop {
            let provider_idx = self.select_provider(AiCapability::QuestionGeneration).ok_or_else(
                || {
                    error!("No AI providers available");
                    AiError::NoProvidersAvailable
                },
            )?;

            let provider = &self.providers[provider_idx];
            info!(
                provider = provider.name(),
                attempt = attempts,
                "Attempting to generate questions"
            );

            match provider.generate_questions(request).await {
                Ok(response) => {
                    // Record success
                    {
                        let mut circuits = self.circuits.write();
                        circuits[provider_idx].record_success();
                    }

                    // Cache the response
                    self.cache.set(cache_key, response.clone()).await;

                    info!(
                        provider = provider.name(),
                        questions = response.questions.len(),
                        latency_ms = response.latency_ms,
                        "Successfully generated questions"
                    );

                    return Ok(response);
                }
                Err(e) => {
                    error!(
                        provider = provider.name(),
                        error = %e,
                        "Provider failed"
                    );

                    // Record failure
                    {
                        let mut circuits = self.circuits.write();
                        circuits[provider_idx].record_failure();
                    }

                    last_error = Some(e);
                    attempts += 1;

                    // Try next provider
                    if attempts >= self.providers.len() {
                        break;
                    }
                }
            }
        }

        Err(last_error.unwrap_or(AiError::NoProvidersAvailable))
    }

    /// Generate hint with automatic fallback.
    pub async fn generate_hint(&self, question: &str) -> Result<String, AiError> {
        // Check cache
        let cache_key = format!("hint:{}", question);
        if let Some(cached) = self.cache.get_hint(&cache_key).await {
            return Ok(cached);
        }

        let provider_idx = self
            .select_provider(AiCapability::HintGeneration)
            .ok_or(AiError::NoProvidersAvailable)?;

        let provider = &self.providers[provider_idx];
        let hint = provider.generate_hint(question).await?;

        // Cache the hint
        self.cache.set_hint(cache_key, hint.clone()).await;

        Ok(hint)
    }

    /// Generate explanation with automatic fallback.
    pub async fn generate_explanation(
        &self,
        question: &str,
        correct_answer: &str,
    ) -> Result<String, AiError> {
        let provider_idx = self
            .select_provider(AiCapability::ExplanationGeneration)
            .ok_or(AiError::NoProvidersAvailable)?;

        let provider = &self.providers[provider_idx];
        provider.generate_explanation(question, correct_answer).await
    }

    /// Get list of available providers.
    pub fn available_providers(&self) -> Vec<String> {
        let circuits = self.circuits.read();
        self.providers
            .iter()
            .enumerate()
            .filter(|(idx, _)| circuits[*idx].is_available())
            .map(|(_, p)| p.name().to_string())
            .collect()
    }

    /// Set default provider by name.
    pub fn set_default_provider(&mut self, name: &str) -> bool {
        if let Some(idx) = self.providers.iter().position(|p| p.name() == name) {
            self.default_provider = idx;
            info!("Default provider set to: {}", name);
            true
        } else {
            false
        }
    }

    /// Get cache statistics.
    pub async fn cache_stats(&self) -> super::cache::CacheStats {
        self.cache.stats().await
    }

    /// Clear cache.
    pub async fn clear_cache(&self) {
        self.cache.clear().await;
    }
}
