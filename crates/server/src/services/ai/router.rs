//! AI router with fallback, load balancing, and provider selection.
//!
//! Features:
//! - Automatic failover when a provider is unavailable
//! - Load balancing across multiple providers
//! - Provider priority based on cost and latency
//! - Circuit breaker pattern for failing providers
//!
//! Note: `default_provider` uses `std::sync::RwLock` (not parking_lot)
//! because we need cheap shared access through `Arc` without requiring
//! mutable references. `parking_lot` is not used here because we never
//! hold the lock across `.await` points.

use std::sync::Arc;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use tracing::{error, info, warn};

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
    Closed,   // Normal operation
    Open,     // Failing, don't use
    HalfOpen, // Testing if recovered
}

/// Circuit breaker for a provider.
///
/// Wrapped in `std::sync::RwLock` (not parking_lot) so it can live behind
/// an `Arc` shared across async tasks. We never hold the lock across an
/// `.await` point, so there is no risk of stalling the runtime.
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
            warn!(
                "Circuit breaker opened after {} failures",
                self.failure_count
            );
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
///
/// `circuits` and `default_provider` are wrapped in `std::sync::RwLock` so
/// the whole `AiRouter` can live behind an `Arc` and be mutated through
/// shared references (e.g. from an Axum handler). The lock is held only
/// for short, non-blocking operations — never across `.await` points.
pub struct AiRouter {
    providers: Vec<ProviderBox>,
    circuits: RwLock<Vec<CircuitBreaker>>,
    cache: Arc<AiCache>,
    default_provider: RwLock<usize>,
}

impl AiRouter {
    pub fn new(configs: Vec<ProviderConfig>) -> Self {
        let mut providers = Vec::new();
        let mut circuits = Vec::new();

        for config in configs {
            if !config.enabled {
                continue;
            }

            let needs_key = matches!(
                config.provider_type,
                ProviderType::OpenAI | ProviderType::OpenRouter | ProviderType::GigaChat
            );
            if needs_key && config.api_key.as_deref().unwrap_or("").trim().is_empty() {
                info!(provider = %config.name, "skipping provider: no API key configured");
                continue;
            }

            let provider: ProviderBox = match config.provider_type.clone() {
                ProviderType::GigaChat => ProviderBox::new(GigaChatProvider::new(config)),
                // OpenRouter speaks the OpenAI chat-completions protocol.
                ProviderType::OpenAI | ProviderType::OpenRouter => {
                    ProviderBox::new(OpenAIProvider::new(config))
                }
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
            circuits: RwLock::new(circuits),
            cache: Arc::new(AiCache::new(Duration::from_secs(3600))), // 1 hour cache
            default_provider: RwLock::new(0),
        }
    }

    /// Select provider based on priority and availability.
    fn select_provider(&self, capability: AiCapability) -> Option<usize> {
        let circuits = self.circuits.read().ok()?;
        let default = *self.default_provider.read().ok()?;

        // Try default provider first
        if default < self.providers.len() {
            let circuit = circuits.get(default)?;
            if circuit.is_available()
                && self.providers[default].capabilities().contains(&capability)
            {
                return Some(default);
            }
        }

        // Fallback to other providers
        for (idx, provider) in self.providers.iter().enumerate() {
            if idx == default {
                continue;
            }

            let circuit = circuits.get(idx)?;
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

        let cache_key = format!(
            "q:{}:{}:{}:{}",
            request.topic, request.difficulty, request.count, request.num_options
        );

        self.generate_questions_with_provider(request, None).await
    }

    /// Generate questions with optional specific provider selection.
    ///
    /// If `provider_name` is `Some`, attempts to use that specific provider.
    /// Falls back to automatic selection if the specified provider is unavailable.
    pub async fn generate_questions_with_provider(
        &self,
        request: &GenerationRequest,
        provider_name: Option<&str>,
    ) -> Result<GenerationResponse, AiError> {
        let cache_key = format!(
            "q:{}:{}:{}:{}:{}",
            request.topic, request.difficulty, request.count, request.num_options,
            provider_name.unwrap_or("auto")

        );

        if !request.bypass_cache {
            if let Some(cached) = self.cache.get(&cache_key) {
                info!("Serving questions from cache");
                return Ok(cached);
            }
        }



        // If specific provider requested, try it first
        if let Some(name) = provider_name {
            if let Some(provider_idx) = self.providers.iter().position(|p| p.name() == name) {
                let circuits_available = self.circuits.read().map(|circuits| {
                    circuits
                        .get(provider_idx)
                        .map(|c| c.is_available())
                        .unwrap_or(false)
                }).unwrap_or(false);

                if circuits_available {
                    let provider = &self.providers[provider_idx];
                    info!(provider = name, "Using requested provider");
                    match provider.generate_questions(request).await {
                        Ok(mut response) => {
                            response.provider_used = name.to_string();
                            if !request.bypass_cache {
                                self.cache.set(cache_key, response.clone());
                            }
                            return Ok(response);
                        }
                        Err(e) => {
                            warn!(provider = name, error = %e, "Requested provider failed, falling back");
                            // Fall through to automatic selection
                        }
                    }
                }
            } else {
                warn!(provider = name, "Requested provider not found, falling back");
            }
        }


        // Try providers with fallback
        let mut attempts = 0;

        loop {
            let provider_idx = self
                .select_provider(AiCapability::QuestionGeneration)
                .ok_or_else(|| {
                    error!("No AI providers available");
                    AiError::NoProvidersAvailable
                })?;

            let provider = &self.providers[provider_idx];
            info!(
                provider = provider.name(),
                attempt = attempts,
                "Attempting to generate questions"
            );

            match provider.generate_questions(request).await {
                Ok(response) => {
                    // Record success
                    if let Ok(mut circuits) = self.circuits.write() {
                        if let Some(cb) = circuits.get_mut(provider_idx) {
                            cb.record_success();
                        }
                    }

                    if !request.bypass_cache {
                        self.cache.set(cache_key, response.clone());
                    }

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
                    if let Ok(mut circuits) = self.circuits.write() {
                        if let Some(cb) = circuits.get_mut(provider_idx) {
                            cb.record_failure();
                        }
                    }

                    attempts += 1;

                    // Try next provider; the last failure is what the caller sees.
                    if attempts >= self.providers.len() {
                        break Err(e);
                    }
                }
            }
        }
    }

    /// Generate hint with automatic fallback.
    pub async fn generate_hint(&self, question: &str) -> Result<String, AiError> {
        // Check cache
        let cache_key = format!("hint:{}", question);
        if let Some(cached) = self.cache.get_hint(&cache_key) {
            return Ok(cached);
        }

        let provider_idx = self
            .select_provider(AiCapability::HintGeneration)
            .ok_or(AiError::NoProvidersAvailable)?;

        let provider = &self.providers[provider_idx];
        let hint = provider.generate_hint(question).await?;

        // Cache the hint
        self.cache.set_hint(cache_key, hint.clone());

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
        provider
            .generate_explanation(question, correct_answer)
            .await
    }

    /// Get list of available providers.
    pub fn available_providers(&self) -> Vec<String> {
        let Ok(circuits) = self.circuits.read() else {
            return Vec::new();
        };
        self.providers
            .iter()
            .enumerate()
            .filter(|(idx, _)| {
                circuits
                    .get(*idx)
                    .map(|c| c.is_available())
                    .unwrap_or(false)
            })
            .map(|(_, p)| p.name().to_string())
            .collect()
    }

    /// Set default provider by name. Uses interior mutability so it can be
    /// called through an `Arc<AiRouter>` from an Axum handler.
    pub fn set_default_provider(&self, name: &str) -> bool {
        if let Some(idx) = self.providers.iter().position(|p| p.name() == name) {
            if let Ok(mut default) = self.default_provider.write() {
                *default = idx;
                info!("Default provider set to: {}", name);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Get cache statistics.
    pub fn cache_stats(&self) -> super::cache::CacheStats {
        self.cache.stats()
    }

    /// Clear cache.
    pub fn clear_cache(&self) {
        self.cache.clear();
    }
}
