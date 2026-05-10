use std::{env, error::Error, fmt};

use async_trait::async_trait;
use rig::{
    client::{CompletionClient, ProviderClient, ProviderClientError},
    completion::{Prompt, PromptError},
    providers::openai,
};

use crate::services::estimate_prompt;

const DEFAULT_OPENAI_MODEL: &str = "gpt-4o-mini";

/// Service abstraction for generating raw estimate text from an LLM.
#[async_trait]
pub trait EstimateTextService: Send + Sync {
    /// Generates raw model output for a given task description.
    async fn generate_estimate_text(
        &self,
        task_description: &str,
    ) -> Result<String, EstimateServiceError>;
}

/// OpenAI-backed implementation of estimate generation using `rig-core`.
#[derive(Debug, Clone)]
pub struct RigOpenAiEstimateService {
    client: openai::Client,
    model: String,
}

impl RigOpenAiEstimateService {
    /// Builds a service using runtime environment configuration.
    ///
    /// Required env vars:
    /// - `OPENAI_API_KEY`
    ///
    /// Optional env vars:
    /// - `OPENAI_MODEL` (defaults to `gpt-4o-mini`)
    pub fn from_env() -> Result<Self, EstimateServiceError> {
        let client =
            openai::Client::from_env().map_err(EstimateServiceError::ClientInitialization)?;
        let model = resolve_openai_model_from_env();

        Ok(Self::new(client, model))
    }

    /// Creates a new service from explicit dependencies.
    #[must_use]
    pub fn new(client: openai::Client, model: impl Into<String>) -> Self {
        Self {
            client,
            model: model.into(),
        }
    }

    /// Returns the configured model name.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }
}

#[async_trait]
impl EstimateTextService for RigOpenAiEstimateService {
    async fn generate_estimate_text(
        &self,
        task_description: &str,
    ) -> Result<String, EstimateServiceError> {
        if task_description.trim().is_empty() {
            return Err(EstimateServiceError::InvalidTaskDescription);
        }

        let prompt_payload = estimate_prompt::build_estimation_prompt(task_description);
        let agent = self.client.agent(self.model.clone()).build();

        agent
            .prompt(&prompt_payload)
            .await
            .map_err(EstimateServiceError::LlmRequest)
    }
}

/// Errors returned by estimate service operations.
#[derive(Debug)]
pub enum EstimateServiceError {
    /// Task description cannot be blank.
    InvalidTaskDescription,
    /// OpenAI client could not be initialized.
    ClientInitialization(ProviderClientError),
    /// LLM request failed.
    LlmRequest(PromptError),
}

impl fmt::Display for EstimateServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTaskDescription => write!(f, "task description must not be blank"),
            Self::ClientInitialization(_) => write!(
                f,
                "failed to initialize OpenAI client from environment configuration"
            ),
            Self::LlmRequest(_) => write!(f, "failed to get response from OpenAI model"),
        }
    }
}

impl Error for EstimateServiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidTaskDescription => None,
            Self::ClientInitialization(source) => Some(source),
            Self::LlmRequest(source) => Some(source),
        }
    }
}

fn resolve_openai_model_from_env() -> String {
    match env::var("OPENAI_MODEL") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => DEFAULT_OPENAI_MODEL.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_openai_model_from_env;
    use serial_test::serial;

    #[test]
    #[serial]
    fn resolves_default_model_when_env_missing() {
        unsafe {
            std::env::remove_var("OPENAI_MODEL");
        }

        assert_eq!(resolve_openai_model_from_env(), "gpt-4o-mini");
    }

    #[test]
    #[serial]
    fn resolves_default_model_when_env_blank() {
        unsafe {
            std::env::set_var("OPENAI_MODEL", "   ");
        }

        assert_eq!(resolve_openai_model_from_env(), "gpt-4o-mini");
    }

    #[test]
    #[serial]
    fn resolves_model_from_env_when_present() {
        unsafe {
            std::env::set_var("OPENAI_MODEL", "gpt-4.1-mini");
        }

        assert_eq!(resolve_openai_model_from_env(), "gpt-4.1-mini");
    }
}
