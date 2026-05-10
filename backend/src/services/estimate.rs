use std::{env, error::Error, fmt, sync::Arc};

use async_trait::async_trait;
use reqwest::Client;
use rig::{
    client::{CompletionClient, ProviderClient, ProviderClientError},
    completion::{Prompt, PromptError},
    providers::openai,
};
use serde::{Deserialize, Serialize};

use crate::services::estimate_prompt;

const DEFAULT_OPENAI_MODEL: &str = "gpt-4o-mini";
const DEFAULT_ELEVENLABS_BASE_URL: &str = "https://api.elevenlabs.io";
const DEFAULT_ELEVENLABS_SIMULATION_TURNS_LIMIT: u32 = 8;
const DEFAULT_ESTIMATE_PROVIDER: &str = "elevenlabs";

/// Service abstraction for generating raw estimate text from an LLM.
#[async_trait]
pub trait EstimateTextService: Send + Sync {
    /// Generates raw model output for a given task description.
    async fn generate_estimate_text(
        &self,
        task_description: &str,
    ) -> Result<String, EstimateServiceError>;
}

pub type EstimateTextServiceFactory =
    dyn Fn() -> Result<Box<dyn EstimateTextService>, EstimateServiceError> + Send + Sync;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EstimateProvider {
    ElevenLabs,
    OpenAi,
}

pub fn build_estimate_text_service_from_env()
-> Result<Box<dyn EstimateTextService>, EstimateServiceError> {
    match resolve_estimate_provider_from_env()? {
        EstimateProvider::ElevenLabs => ElevenLabsEstimateService::from_env()
            .map(|service| Box::new(service) as Box<dyn EstimateTextService>),
        EstimateProvider::OpenAi => RigOpenAiEstimateService::from_env()
            .map(|service| Box::new(service) as Box<dyn EstimateTextService>),
    }
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

#[derive(Clone)]
pub struct ElevenLabsEstimateService {
    client: Arc<dyn ElevenLabsAgentsClient>,
    config: ElevenLabsAgentsConfig,
}

impl ElevenLabsEstimateService {
    /// Builds a service using runtime environment configuration.
    ///
    /// Required env vars:
    /// - `ELEVENLABS_API_KEY`
    /// - `ELEVENLABS_AGENT_ID`
    ///
    /// Optional env vars:
    /// - `ELEVENLABS_BASE_URL` (defaults to `https://api.elevenlabs.io`)
    /// - `ELEVENLABS_SIMULATION_TURNS_LIMIT` (defaults to `8`)
    pub fn from_env() -> Result<Self, EstimateServiceError> {
        let config = ElevenLabsAgentsConfig::from_env()?;
        println!("{:?}", config);
        let http_client = Client::builder()
            .build()
            .map_err(EstimateServiceError::ElevenLabsClientInitialization)?;

        Ok(Self::new(
            Arc::new(ReqwestElevenLabsAgentsClient::new(http_client)),
            config,
        ))
    }

    #[must_use]
    fn new(client: Arc<dyn ElevenLabsAgentsClient>, config: ElevenLabsAgentsConfig) -> Self {
        Self { client, config }
    }

    #[must_use]
    pub fn config(&self) -> &ElevenLabsAgentsConfig {
        &self.config
    }
}

#[async_trait]
impl EstimateTextService for ElevenLabsEstimateService {
    async fn generate_estimate_text(
        &self,
        task_description: &str,
    ) -> Result<String, EstimateServiceError> {
        if task_description.trim().is_empty() {
            return Err(EstimateServiceError::InvalidTaskDescription);
        }

        let prompt_payload = estimate_prompt::build_estimation_prompt(task_description);
        let request = ElevenLabsSimulateConversationRequest::from_prompt(
            prompt_payload,
            self.config.new_turns_limit,
        );
        let response = self
            .client
            .simulate_conversation(&self.config, &request)
            .await?;

        extract_agent_message(&response).ok_or(EstimateServiceError::ElevenLabsEmptyResponse)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElevenLabsAgentsConfig {
    api_key: String,
    agent_id: String,
    base_url: String,
    new_turns_limit: u32,
}

impl ElevenLabsAgentsConfig {
    pub fn from_env() -> Result<Self, EstimateServiceError> {
        let api_key = required_non_empty_env("ELEVENLABS_API_KEY")?;
        let agent_id = required_non_empty_env("ELEVENLABS_AGENT_ID")?;
        let base_url = resolve_elevenlabs_base_url_from_env();
        let new_turns_limit = resolve_elevenlabs_new_turns_limit_from_env()?;

        Ok(Self {
            api_key,
            agent_id,
            base_url,
            new_turns_limit,
        })
    }

    #[must_use]
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    #[must_use]
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }

    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    #[must_use]
    pub fn new_turns_limit(&self) -> u32 {
        self.new_turns_limit
    }
}

#[async_trait]
trait ElevenLabsAgentsClient: Send + Sync {
    async fn simulate_conversation(
        &self,
        config: &ElevenLabsAgentsConfig,
        request: &ElevenLabsSimulateConversationRequest,
    ) -> Result<ElevenLabsSimulateConversationResponse, EstimateServiceError>;
}

#[derive(Debug, Clone)]
struct ReqwestElevenLabsAgentsClient {
    client: Client,
}

impl ReqwestElevenLabsAgentsClient {
    #[must_use]
    fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ElevenLabsAgentsClient for ReqwestElevenLabsAgentsClient {
    async fn simulate_conversation(
        &self,
        config: &ElevenLabsAgentsConfig,
        request: &ElevenLabsSimulateConversationRequest,
    ) -> Result<ElevenLabsSimulateConversationResponse, EstimateServiceError> {
        let url = format!(
            "{}/v1/convai/agents/{}/simulate-conversation",
            config.base_url().trim_end_matches('/'),
            config.agent_id()
        );

        let response = self
            .client
            .post(url)
            .header("xi-api-key", config.api_key())
            .json(request)
            .send()
            .await
            .map_err(EstimateServiceError::ElevenLabsRequestFailed)?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "failed to read ElevenLabs error body".to_string());
            return Err(EstimateServiceError::ElevenLabsRequestUnsuccessful {
                status_code: status.as_u16(),
                body,
            });
        }

        response
            .json::<ElevenLabsSimulateConversationResponse>()
            .await
            .map_err(EstimateServiceError::ElevenLabsRequestFailed)
    }
}

#[derive(Debug, Clone, Serialize)]
struct ElevenLabsSimulateConversationRequest {
    simulation_specification: ElevenLabsSimulationSpecification,
    new_turns_limit: u32,
}

impl ElevenLabsSimulateConversationRequest {
    fn from_prompt(prompt: String, new_turns_limit: u32) -> Self {
        Self {
            simulation_specification: ElevenLabsSimulationSpecification {
                simulated_user_config: ElevenLabsSimulatedUserConfig {
                    first_message: String::new(),
                    language: "en".to_string(),
                    prompt: ElevenLabsPromptConfig {
                        prompt: "Act as the user requesting a software estimate.".to_string(),
                    },
                },
                partial_conversation_history: vec![ElevenLabsConversationTurnInput {
                    role: "user".to_string(),
                    message: prompt,
                    time_in_call_secs: 0,
                }],
            },
            new_turns_limit,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct ElevenLabsSimulationSpecification {
    simulated_user_config: ElevenLabsSimulatedUserConfig,
    partial_conversation_history: Vec<ElevenLabsConversationTurnInput>,
}

#[derive(Debug, Clone, Serialize)]
struct ElevenLabsSimulatedUserConfig {
    first_message: String,
    language: String,
    prompt: ElevenLabsPromptConfig,
}

#[derive(Debug, Clone, Serialize)]
struct ElevenLabsPromptConfig {
    prompt: String,
}

#[derive(Debug, Clone, Serialize)]
struct ElevenLabsConversationTurnInput {
    role: String,
    message: String,
    time_in_call_secs: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct ElevenLabsSimulateConversationResponse {
    simulated_conversation: Vec<ElevenLabsConversationTurnOutput>,
}

#[derive(Debug, Clone, Deserialize)]
struct ElevenLabsConversationTurnOutput {
    role: String,
    message: Option<String>,
}

fn extract_agent_message(response: &ElevenLabsSimulateConversationResponse) -> Option<String> {
    response
        .simulated_conversation
        .iter()
        .rev()
        .find(|turn| turn.role == "agent")
        .and_then(|turn| turn.message.as_deref())
        .map(str::trim)
        .filter(|message| !message.is_empty())
        .map(ToOwned::to_owned)
}

/// Errors returned by estimate service operations.
#[derive(Debug)]
pub enum EstimateServiceError {
    /// Task description cannot be blank.
    InvalidTaskDescription,
    /// OpenAI client could not be initialized.
    ClientInitialization(ProviderClientError),
    /// OpenAI request failed.
    LlmRequest(PromptError),
    /// Required service configuration key is missing or blank.
    MissingConfiguration { key: &'static str },
    /// Service configuration value is invalid.
    InvalidConfiguration { key: &'static str, message: String },
    /// ElevenLabs HTTP client could not be initialized.
    ElevenLabsClientInitialization(reqwest::Error),
    /// ElevenLabs request failed before receiving a successful response body.
    ElevenLabsRequestFailed(reqwest::Error),
    /// ElevenLabs request completed with a non-success HTTP status.
    ElevenLabsRequestUnsuccessful { status_code: u16, body: String },
    /// ElevenLabs response did not contain a usable agent message.
    ElevenLabsEmptyResponse,
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
            Self::MissingConfiguration { key } => {
                write!(f, "missing required configuration key: {key}")
            }
            Self::InvalidConfiguration { key, message } => {
                write!(f, "invalid configuration for {key}: {message}")
            }
            Self::ElevenLabsClientInitialization(_) => write!(
                f,
                "failed to initialize ElevenLabs client from environment configuration"
            ),
            Self::ElevenLabsRequestFailed(_) => {
                write!(f, "failed to get response from ElevenLabs Agents API")
            }
            Self::ElevenLabsRequestUnsuccessful { status_code, .. } => write!(
                f,
                "ElevenLabs Agents API responded with non-success status: {status_code}"
            ),
            Self::ElevenLabsEmptyResponse => {
                write!(
                    f,
                    "ElevenLabs Agents API response did not contain an agent message"
                )
            }
        }
    }
}

impl Error for EstimateServiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidTaskDescription
            | Self::MissingConfiguration { .. }
            | Self::InvalidConfiguration { .. }
            | Self::ElevenLabsRequestUnsuccessful { .. }
            | Self::ElevenLabsEmptyResponse => None,
            Self::ClientInitialization(source) => Some(source),
            Self::LlmRequest(source) => Some(source),
            Self::ElevenLabsClientInitialization(source) => Some(source),
            Self::ElevenLabsRequestFailed(source) => Some(source),
        }
    }
}

fn resolve_openai_model_from_env() -> String {
    match env::var("OPENAI_MODEL") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => DEFAULT_OPENAI_MODEL.to_string(),
    }
}

fn resolve_estimate_provider_from_env() -> Result<EstimateProvider, EstimateServiceError> {
    let value = match env::var("ESTIMATE_PROVIDER") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => return Ok(EstimateProvider::ElevenLabs),
    };

    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "elevenlabs" => Ok(EstimateProvider::ElevenLabs),
        "openai" => Ok(EstimateProvider::OpenAi),
        _ => Err(EstimateServiceError::InvalidConfiguration {
            key: "ESTIMATE_PROVIDER",
            message: format!(
                "unsupported provider `{value}`; expected `{DEFAULT_ESTIMATE_PROVIDER}` or `openai`"
            ),
        }),
    }
}

fn required_non_empty_env(key: &'static str) -> Result<String, EstimateServiceError> {
    match env::var(key) {
        Ok(value) if !value.trim().is_empty() => Ok(value),
        _ => Err(EstimateServiceError::MissingConfiguration { key }),
    }
}

fn resolve_elevenlabs_base_url_from_env() -> String {
    match env::var("ELEVENLABS_BASE_URL") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => DEFAULT_ELEVENLABS_BASE_URL.to_string(),
    }
}

fn resolve_elevenlabs_new_turns_limit_from_env() -> Result<u32, EstimateServiceError> {
    let value = match env::var("ELEVENLABS_SIMULATION_TURNS_LIMIT") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => return Ok(DEFAULT_ELEVENLABS_SIMULATION_TURNS_LIMIT),
    };

    let parsed = value
        .parse::<u32>()
        .map_err(|_| EstimateServiceError::InvalidConfiguration {
            key: "ELEVENLABS_SIMULATION_TURNS_LIMIT",
            message: "must be a positive integer".to_string(),
        })?;

    if parsed == 0 {
        return Err(EstimateServiceError::InvalidConfiguration {
            key: "ELEVENLABS_SIMULATION_TURNS_LIMIT",
            message: "must be greater than 0".to_string(),
        });
    }

    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        vec,
    };

    use super::{
        ElevenLabsAgentsClient, ElevenLabsAgentsConfig, ElevenLabsConversationTurnOutput,
        ElevenLabsEstimateService, ElevenLabsSimulateConversationResponse, EstimateProvider,
        EstimateServiceError, EstimateTextService, build_estimate_text_service_from_env,
        resolve_elevenlabs_base_url_from_env, resolve_elevenlabs_new_turns_limit_from_env,
        resolve_estimate_provider_from_env, resolve_openai_model_from_env,
    };
    use async_trait::async_trait;
    use serial_test::serial;

    struct StubElevenLabsAgentsClient {
        response:
            Mutex<Option<Result<ElevenLabsSimulateConversationResponse, EstimateServiceError>>>,
        last_turn_limit: Mutex<Option<u32>>,
        last_user_message: Mutex<Option<String>>,
    }

    impl StubElevenLabsAgentsClient {
        fn with_response(
            response: Result<ElevenLabsSimulateConversationResponse, EstimateServiceError>,
        ) -> Self {
            Self {
                response: Mutex::new(Some(response)),
                last_turn_limit: Mutex::new(None),
                last_user_message: Mutex::new(None),
            }
        }

        fn captured_turn_limit(&self) -> Option<u32> {
            self.last_turn_limit
                .lock()
                .expect("turn limit lock poisoned")
                .to_owned()
        }

        fn captured_user_message(&self) -> Option<String> {
            self.last_user_message
                .lock()
                .expect("user message lock poisoned")
                .clone()
        }
    }

    #[async_trait]
    impl ElevenLabsAgentsClient for StubElevenLabsAgentsClient {
        async fn simulate_conversation(
            &self,
            _config: &ElevenLabsAgentsConfig,
            request: &super::ElevenLabsSimulateConversationRequest,
        ) -> Result<ElevenLabsSimulateConversationResponse, EstimateServiceError> {
            *self
                .last_turn_limit
                .lock()
                .expect("turn limit lock poisoned") = Some(request.new_turns_limit);

            *self
                .last_user_message
                .lock()
                .expect("user message lock poisoned") = request
                .simulation_specification
                .partial_conversation_history
                .first()
                .map(|turn| turn.message.clone());

            self.response
                .lock()
                .expect("response lock poisoned")
                .take()
                .expect("response should be set")
        }
    }

    fn elevenlabs_config() -> ElevenLabsAgentsConfig {
        ElevenLabsAgentsConfig {
            api_key: "test-api-key".to_string(),
            agent_id: "agent_123".to_string(),
            base_url: "https://api.elevenlabs.io".to_string(),
            new_turns_limit: 7,
        }
    }

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
    fn resolves_default_provider_when_env_missing() {
        unsafe {
            std::env::remove_var("ESTIMATE_PROVIDER");
        }

        assert!(matches!(
            resolve_estimate_provider_from_env(),
            Ok(EstimateProvider::ElevenLabs)
        ));
    }

    #[test]
    #[serial]
    fn resolves_openai_provider_when_explicitly_requested() {
        unsafe {
            std::env::set_var("ESTIMATE_PROVIDER", "openai");
        }

        assert!(matches!(
            resolve_estimate_provider_from_env(),
            Ok(EstimateProvider::OpenAi)
        ));
    }

    #[test]
    #[serial]
    fn build_service_uses_elevenlabs_by_default() {
        unsafe {
            std::env::remove_var("ESTIMATE_PROVIDER");
            std::env::set_var("ELEVENLABS_API_KEY", "el_key");
            std::env::set_var("ELEVENLABS_AGENT_ID", "agent_123");
            std::env::remove_var("OPENAI_API_KEY");
        }

        let result = build_estimate_text_service_from_env();
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn build_service_requires_elevenlabs_env_when_default_provider_used() {
        unsafe {
            std::env::remove_var("ESTIMATE_PROVIDER");
            std::env::remove_var("ELEVENLABS_API_KEY");
            std::env::set_var("ELEVENLABS_AGENT_ID", "agent_123");
        }

        let result = build_estimate_text_service_from_env();
        assert!(matches!(
            result,
            Err(EstimateServiceError::MissingConfiguration {
                key: "ELEVENLABS_API_KEY"
            })
        ));
    }

    #[test]
    #[serial]
    fn build_service_switches_to_openai_when_provider_is_openai() {
        unsafe {
            std::env::set_var("ESTIMATE_PROVIDER", "openai");
            std::env::set_var("ELEVENLABS_API_KEY", "el_key");
            std::env::set_var("ELEVENLABS_AGENT_ID", "agent_123");
            std::env::remove_var("OPENAI_API_KEY");
        }

        let result = build_estimate_text_service_from_env();
        assert!(matches!(
            result,
            Err(EstimateServiceError::ClientInitialization(_))
        ));
    }

    #[test]
    #[serial]
    fn rejects_unknown_provider() {
        unsafe {
            std::env::set_var("ESTIMATE_PROVIDER", "not-a-provider");
        }

        let result = resolve_estimate_provider_from_env();
        assert!(matches!(
            result,
            Err(EstimateServiceError::InvalidConfiguration {
                key: "ESTIMATE_PROVIDER",
                ..
            })
        ));
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

    #[test]
    #[serial]
    fn resolves_default_elevenlabs_base_url_when_env_missing() {
        unsafe {
            std::env::remove_var("ELEVENLABS_BASE_URL");
        }

        assert_eq!(
            resolve_elevenlabs_base_url_from_env(),
            "https://api.elevenlabs.io"
        );
    }

    #[test]
    #[serial]
    fn resolves_elevenlabs_base_url_from_env_when_present() {
        unsafe {
            std::env::set_var("ELEVENLABS_BASE_URL", "https://example.elevenlabs.test");
        }

        assert_eq!(
            resolve_elevenlabs_base_url_from_env(),
            "https://example.elevenlabs.test"
        );
    }

    #[test]
    #[serial]
    fn resolves_default_elevenlabs_turn_limit_when_env_missing() {
        unsafe {
            std::env::remove_var("ELEVENLABS_SIMULATION_TURNS_LIMIT");
        }

        assert!(matches!(
            resolve_elevenlabs_new_turns_limit_from_env(),
            Ok(8)
        ));
    }

    #[test]
    #[serial]
    fn resolves_elevenlabs_turn_limit_from_env_when_present() {
        unsafe {
            std::env::set_var("ELEVENLABS_SIMULATION_TURNS_LIMIT", "12");
        }

        assert!(matches!(
            resolve_elevenlabs_new_turns_limit_from_env(),
            Ok(12)
        ));
    }

    #[test]
    #[serial]
    fn rejects_non_numeric_elevenlabs_turn_limit() {
        unsafe {
            std::env::set_var("ELEVENLABS_SIMULATION_TURNS_LIMIT", "abc");
        }

        let result = resolve_elevenlabs_new_turns_limit_from_env();
        assert!(matches!(
            result,
            Err(EstimateServiceError::InvalidConfiguration {
                key: "ELEVENLABS_SIMULATION_TURNS_LIMIT",
                ..
            })
        ));
    }

    #[test]
    #[serial]
    fn rejects_zero_elevenlabs_turn_limit() {
        unsafe {
            std::env::set_var("ELEVENLABS_SIMULATION_TURNS_LIMIT", "0");
        }

        let result = resolve_elevenlabs_new_turns_limit_from_env();
        assert!(matches!(
            result,
            Err(EstimateServiceError::InvalidConfiguration {
                key: "ELEVENLABS_SIMULATION_TURNS_LIMIT",
                ..
            })
        ));
    }

    #[test]
    #[serial]
    fn elevenlabs_config_requires_api_key() {
        unsafe {
            std::env::remove_var("ELEVENLABS_API_KEY");
            std::env::set_var("ELEVENLABS_AGENT_ID", "agent_1");
        }

        let result = ElevenLabsAgentsConfig::from_env();
        assert!(matches!(
            result,
            Err(EstimateServiceError::MissingConfiguration {
                key: "ELEVENLABS_API_KEY"
            })
        ));
    }

    #[test]
    #[serial]
    fn elevenlabs_config_requires_agent_id() {
        unsafe {
            std::env::set_var("ELEVENLABS_API_KEY", "key_1");
            std::env::remove_var("ELEVENLABS_AGENT_ID");
        }

        let result = ElevenLabsAgentsConfig::from_env();
        assert!(matches!(
            result,
            Err(EstimateServiceError::MissingConfiguration {
                key: "ELEVENLABS_AGENT_ID"
            })
        ));
    }

    #[tokio::test]
    async fn elevenlabs_service_returns_last_agent_message() {
        let client = Arc::new(StubElevenLabsAgentsClient::with_response(Ok(
            ElevenLabsSimulateConversationResponse {
                simulated_conversation: vec![
                    ElevenLabsConversationTurnOutput {
                        role: "user".to_string(),
                        message: Some("user prompt".to_string()),
                    },
                    ElevenLabsConversationTurnOutput {
                        role: "agent".to_string(),
                        message: Some(
                            "{\"price_sol\":380,\"complexity\":3,\"rationale\":\"ok\"}".to_string(),
                        ),
                    },
                ],
            },
        )));

        let service = ElevenLabsEstimateService::new(client.clone(), elevenlabs_config());

        let output = service
            .generate_estimate_text("Dodaj endpoint API")
            .await
            .expect("service should return text output");

        assert_eq!(
            output,
            "{\"price_sol\":380,\"complexity\":3,\"rationale\":\"ok\"}"
        );
        assert_eq!(client.captured_turn_limit(), Some(7));
        assert!(
            client
                .captured_user_message()
                .is_some_and(|value| value.contains("Dodaj endpoint API"))
        );
    }

    #[tokio::test]
    async fn elevenlabs_service_returns_error_when_no_agent_message_present() {
        let client = Arc::new(StubElevenLabsAgentsClient::with_response(Ok(
            ElevenLabsSimulateConversationResponse {
                simulated_conversation: vec![ElevenLabsConversationTurnOutput {
                    role: "user".to_string(),
                    message: Some("hello".to_string()),
                }],
            },
        )));

        let service = ElevenLabsEstimateService::new(client, elevenlabs_config());
        let result = service.generate_estimate_text("Dodaj endpoint API").await;

        assert!(matches!(
            result,
            Err(EstimateServiceError::ElevenLabsEmptyResponse)
        ));
    }

    #[tokio::test]
    async fn elevenlabs_service_rejects_blank_task_description() {
        let client = Arc::new(StubElevenLabsAgentsClient::with_response(Ok(
            ElevenLabsSimulateConversationResponse {
                simulated_conversation: vec![],
            },
        )));

        let service = ElevenLabsEstimateService::new(client, elevenlabs_config());
        let result = service.generate_estimate_text("   ").await;

        assert!(matches!(
            result,
            Err(EstimateServiceError::InvalidTaskDescription)
        ));
    }
}
