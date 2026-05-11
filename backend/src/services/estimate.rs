use std::{env, error::Error, fmt, sync::Arc};

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use rig::{
    client::{CompletionClient, ProviderClient, ProviderClientError},
    completion::{Prompt, PromptError},
    providers::openai,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time::{Duration, timeout};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, client::IntoClientRequest, http::HeaderValue},
};

use crate::services::estimate_prompt;

const DEFAULT_OPENAI_MODEL: &str = "gpt-4o-mini";
const DEFAULT_ELEVENLABS_BASE_URL: &str = "https://api.elevenlabs.io";
const DEFAULT_ELEVENLABS_CONVERSATION_WS_URL: &str =
    "wss://api.elevenlabs.io/v1/convai/conversation";
const DEFAULT_ELEVENLABS_SIMULATION_TURNS_LIMIT: u32 = 8;
const DEFAULT_ELEVENLABS_CONVERSATION_TIMEOUT_SECS: u64 = 600;
const DEFAULT_ESTIMATE_PROVIDER: &str = "elevenlabs";
const DEFAULT_ELEVENLABS_SIMULATED_USER_LANGUAGE: &str = "en";

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

        let user_message = if self.config.use_backend_prompt() {
            estimate_prompt::build_estimation_prompt(task_description)
        } else {
            task_description.to_string()
        };
        let request =
            ElevenLabsSimulateConversationRequest::from_prompt(&self.config, user_message);
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
    conversation_ws_url: String,
    conversation_timeout_secs: u64,
    new_turns_limit: u32,
    simulated_user_language: String,
    simulated_user_prompt: Option<String>,
    simulated_user_llm: Option<String>,
    use_backend_prompt: bool,
}

impl ElevenLabsAgentsConfig {
    pub fn from_env() -> Result<Self, EstimateServiceError> {
        let api_key = required_non_empty_env("ELEVENLABS_API_KEY")?;
        let agent_id = required_non_empty_env("ELEVENLABS_AGENT_ID")?;
        let base_url = resolve_elevenlabs_base_url_from_env();
        let conversation_ws_url = resolve_elevenlabs_conversation_ws_url_from_env();
        let conversation_timeout_secs = resolve_elevenlabs_conversation_timeout_secs_from_env()?;
        let new_turns_limit = resolve_elevenlabs_new_turns_limit_from_env()?;
        let simulated_user_language = resolve_elevenlabs_simulated_user_language_from_env();
        let simulated_user_prompt = resolve_elevenlabs_simulated_user_prompt_from_env();
        let simulated_user_llm = resolve_elevenlabs_simulated_user_llm_from_env();
        let use_backend_prompt = resolve_elevenlabs_use_backend_prompt_from_env()?;

        Ok(Self {
            api_key,
            agent_id,
            base_url,
            conversation_ws_url,
            conversation_timeout_secs,
            new_turns_limit,
            simulated_user_language,
            simulated_user_prompt,
            simulated_user_llm,
            use_backend_prompt,
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
    pub fn conversation_ws_url(&self) -> &str {
        &self.conversation_ws_url
    }

    #[must_use]
    pub fn conversation_timeout_secs(&self) -> u64 {
        self.conversation_timeout_secs
    }

    #[must_use]
    pub fn new_turns_limit(&self) -> u32 {
        self.new_turns_limit
    }

    #[must_use]
    pub fn simulated_user_language(&self) -> &str {
        &self.simulated_user_language
    }

    #[must_use]
    pub fn simulated_user_prompt(&self) -> Option<&str> {
        self.simulated_user_prompt.as_deref()
    }

    #[must_use]
    pub fn simulated_user_llm(&self) -> Option<&str> {
        self.simulated_user_llm.as_deref()
    }

    #[must_use]
    pub fn use_backend_prompt(&self) -> bool {
        self.use_backend_prompt
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

    async fn simulate_conversation_with_ws(
        &self,
        config: &ElevenLabsAgentsConfig,
        request: &ElevenLabsSimulateConversationRequest,
    ) -> Result<ElevenLabsSimulateConversationResponse, EstimateServiceError> {
        let ws_url = build_conversation_ws_url(config);
        let mut ws_request = ws_url.into_client_request().map_err(|error| {
            EstimateServiceError::InvalidConfiguration {
                key: "ELEVENLABS_CONVERSATION_WS_URL",
                message: format!("failed to build WebSocket request: {error}"),
            }
        })?;
        let api_key = HeaderValue::from_str(config.api_key()).map_err(|error| {
            EstimateServiceError::InvalidConfiguration {
                key: "ELEVENLABS_API_KEY",
                message: format!("invalid API key header value: {error}"),
            }
        })?;
        ws_request.headers_mut().insert("xi-api-key", api_key);

        let (mut ws_stream, _) = connect_async(ws_request).await.map_err(|error| {
            EstimateServiceError::ElevenLabsRequestUnsuccessful {
                status_code: 0,
                body: format!("failed to connect to ElevenLabs conversation websocket: {error}"),
            }
        })?;

        let init_payload = build_conversation_init_event(config, request);
        send_ws_json(&mut ws_stream, &init_payload).await?;

        let user_message = request
            .simulation_specification
            .partial_conversation_history
            .first()
            .map(|turn| turn.message.clone())
            .ok_or(EstimateServiceError::ElevenLabsEmptyResponse)?;
        let user_message_alias = user_message.clone();
        let user_payload = serde_json::json!({
            "type": "user_message",
            "text": user_message,
            "user_message": user_message_alias
        });
        send_ws_json(&mut ws_stream, &user_payload).await?;

        let agent_message = read_agent_message_from_ws(&mut ws_stream, config).await?;

        Ok(ElevenLabsSimulateConversationResponse {
            simulated_conversation: vec![ElevenLabsConversationTurnOutput {
                role: "agent".to_string(),
                message: Some(agent_message),
            }],
        })
    }

    async fn simulate_conversation_with_rest(
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

#[async_trait]
impl ElevenLabsAgentsClient for ReqwestElevenLabsAgentsClient {
    async fn simulate_conversation(
        &self,
        config: &ElevenLabsAgentsConfig,
        request: &ElevenLabsSimulateConversationRequest,
    ) -> Result<ElevenLabsSimulateConversationResponse, EstimateServiceError> {
        match self.simulate_conversation_with_ws(config, request).await {
            Ok(response) => Ok(response),
            Err(ws_error) => {
                tracing::warn!(
                    "ElevenLabs conversation websocket failed; retrying with simulate-conversation REST fallback: {ws_error}"
                );
                self.simulate_conversation_with_rest(config, request).await
            }
        }
    }
}

fn build_conversation_ws_url(config: &ElevenLabsAgentsConfig) -> String {
    let base_url = config.conversation_ws_url().trim_end_matches('/');
    if base_url.contains('?') {
        format!("{base_url}&agent_id={}", config.agent_id())
    } else {
        format!("{base_url}?agent_id={}", config.agent_id())
    }
}

fn build_conversation_init_event(
    config: &ElevenLabsAgentsConfig,
    request: &ElevenLabsSimulateConversationRequest,
) -> Value {
    let mut payload = serde_json::json!({
        "type": "conversation_initiation_client_data",
        "conversation_initiation_client_data": {
            "agent_id": config.agent_id(),
            "new_turns_limit": request.new_turns_limit
        }
    });

    if let Some(prompt_config) = request
        .simulation_specification
        .simulated_user_config
        .prompt
        .as_ref()
    {
        payload["conversation_initiation_client_data"]["conversation_config_override"] = serde_json::json!({
            "agent": {
                "prompt": {
                    "prompt": prompt_config.prompt,
                    "llm": prompt_config.llm
                }
            }
        });
    }

    payload
}

async fn send_ws_json(
    ws_stream: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    payload: &Value,
) -> Result<(), EstimateServiceError> {
    let text = payload.to_string();
    ws_stream
        .send(Message::Text(text.into()))
        .await
        .map_err(
            |error| EstimateServiceError::ElevenLabsRequestUnsuccessful {
                status_code: 0,
                body: format!("failed to send conversation websocket message: {error}"),
            },
        )
}

async fn read_agent_message_from_ws(
    ws_stream: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    config: &ElevenLabsAgentsConfig,
) -> Result<String, EstimateServiceError> {
    let read_until_message = async {
        let mut partial_text = String::new();
        let mut latest_non_estimate_response: Option<String> = None;

        while let Some(next_message) = ws_stream.next().await {
            let message = next_message.map_err(|error| {
                EstimateServiceError::ElevenLabsRequestUnsuccessful {
                    status_code: 0,
                    body: format!("failed while reading conversation websocket: {error}"),
                }
            })?;

            match message {
                Message::Text(text) => {
                    let payload = serde_json::from_str::<Value>(&text).map_err(|error| {
                        EstimateServiceError::ElevenLabsRequestUnsuccessful {
                            status_code: 0,
                            body: format!("invalid websocket JSON payload: {error}"),
                        }
                    })?;

                    if let Some(event_type) = payload.get("type").and_then(Value::as_str)
                        && event_type == "ping"
                    {
                        if let Some(event_id) = payload
                            .get("ping_event")
                            .and_then(|ping| ping.get("event_id"))
                            .and_then(Value::as_str)
                        {
                            let pong = serde_json::json!({
                                "type": "pong",
                                "event_id": event_id
                            });
                            send_ws_json(ws_stream, &pong).await?;
                        }
                        continue;
                    }

                    if let Some(response) = extract_agent_text_from_ws_event(&payload) {
                        if let Some(estimate_json) = extract_estimate_json_from_raw(&response) {
                            return Ok(estimate_json);
                        }
                        latest_non_estimate_response = Some(response);
                    }

                    if let Some(chunk) = extract_agent_text_chunk_from_ws_event(&payload) {
                        partial_text.push_str(&chunk);
                        if let Some(estimate_json) = extract_estimate_json_from_raw(&partial_text) {
                            return Ok(estimate_json);
                        }
                    }
                }
                Message::Binary(binary) => {
                    if let Ok(text) = String::from_utf8(binary.to_vec())
                        && let Ok(payload) = serde_json::from_str::<Value>(&text)
                        && let Some(response) = extract_agent_text_from_ws_event(&payload)
                    {
                        if let Some(estimate_json) = extract_estimate_json_from_raw(&response) {
                            return Ok(estimate_json);
                        }
                        latest_non_estimate_response = Some(response);
                    }
                }
                Message::Close(_) => break,
                Message::Ping(payload) => {
                    ws_stream
                        .send(Message::Pong(payload))
                        .await
                        .map_err(
                            |error| EstimateServiceError::ElevenLabsRequestUnsuccessful {
                                status_code: 0,
                                body: format!("failed to send websocket pong frame: {error}"),
                            },
                        )?;
                }
                Message::Pong(_) | Message::Frame(_) => {}
            }
        }

        if let Some(estimate_json) = extract_estimate_json_from_raw(&partial_text) {
            Ok(estimate_json)
        } else if let Some(latest_non_estimate_response) = latest_non_estimate_response {
            Err(EstimateServiceError::ElevenLabsRequestUnsuccessful {
                status_code: 0,
                body: format!(
                    "conversation websocket returned non-estimate response: {latest_non_estimate_response}"
                ),
            })
        } else if partial_text.trim().is_empty() {
            Err(EstimateServiceError::ElevenLabsEmptyResponse)
        } else {
            Err(EstimateServiceError::ElevenLabsRequestUnsuccessful {
                status_code: 0,
                body: format!(
                    "conversation websocket returned partial non-estimate response: {partial_text}"
                ),
            })
        }
    };

    timeout(
        Duration::from_secs(config.conversation_timeout_secs()),
        read_until_message,
    )
    .await
    .map_err(|_| EstimateServiceError::ElevenLabsRequestUnsuccessful {
        status_code: 0,
        body: format!(
            "conversation websocket timed out after {} seconds",
            config.conversation_timeout_secs()
        ),
    })?
}

fn extract_estimate_json_from_raw(raw: &str) -> Option<String> {
    let candidates = candidate_json_strings(raw);

    candidates.into_iter().find_map(|candidate| {
        let parsed = serde_json::from_str::<Value>(&candidate).ok()?;
        if parsed.get("tasks").is_some_and(|tasks| tasks.is_array()) {
            Some(candidate)
        } else {
            None
        }
    })
}

fn candidate_json_strings(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let mut candidates = vec![trimmed.to_string()];

    if trimmed.starts_with("```")
        && let Some(stripped) = trimmed.strip_prefix("```")
        && let Some(newline_idx) = stripped.find('\n')
    {
        let fenced_body = stripped[(newline_idx + 1)..]
            .strip_suffix("```")
            .unwrap_or(&stripped[(newline_idx + 1)..])
            .trim();
        if !fenced_body.is_empty() {
            candidates.push(fenced_body.to_string());
        }
    }

    if let (Some(start_idx), Some(end_idx)) = (trimmed.find('{'), trimmed.rfind('}'))
        && start_idx < end_idx
    {
        let object_candidate = trimmed[start_idx..=end_idx].trim();
        if !object_candidate.is_empty() {
            candidates.push(object_candidate.to_string());
        }
    }

    candidates
}

fn extract_agent_text_from_ws_event(payload: &Value) -> Option<String> {
    let response = payload
        .get("agent_response_event")
        .and_then(|event| event.get("agent_response"))
        .and_then(Value::as_str)
        .or_else(|| payload.get("agent_response").and_then(Value::as_str))
        .map(str::trim)
        .filter(|text| !text.is_empty())?;

    Some(response.to_string())
}

fn extract_agent_text_chunk_from_ws_event(payload: &Value) -> Option<String> {
    payload
        .get("text_response_part")
        .and_then(|part| part.get("text"))
        .and_then(Value::as_str)
        .or_else(|| {
            payload
                .get("agent_response_event")
                .and_then(|event| event.get("agent_response"))
                .and_then(Value::as_str)
        })
        .map(ToOwned::to_owned)
}

#[derive(Debug, Clone, Serialize)]
struct ElevenLabsSimulateConversationRequest {
    simulation_specification: ElevenLabsSimulationSpecification,
    new_turns_limit: u32,
}

impl ElevenLabsSimulateConversationRequest {
    fn from_prompt(config: &ElevenLabsAgentsConfig, prompt_payload: String) -> Self {
        Self {
            simulation_specification: ElevenLabsSimulationSpecification {
                simulated_user_config: ElevenLabsSimulatedUserConfig {
                    first_message: String::new(),
                    language: config.simulated_user_language().to_string(),
                    prompt: config
                        .simulated_user_prompt()
                        .map(|prompt| ElevenLabsPromptConfig {
                            prompt: prompt.to_string(),
                            llm: config.simulated_user_llm().map(ToOwned::to_owned),
                        }),
                },
                partial_conversation_history: vec![ElevenLabsConversationTurnInput {
                    role: "user".to_string(),
                    message: prompt_payload,
                    time_in_call_secs: 0,
                }],
            },
            new_turns_limit: config.new_turns_limit(),
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
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt: Option<ElevenLabsPromptConfig>,
}

#[derive(Debug, Clone, Serialize)]
struct ElevenLabsPromptConfig {
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    llm: Option<String>,
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

fn resolve_elevenlabs_conversation_ws_url_from_env() -> String {
    match env::var("ELEVENLABS_CONVERSATION_WS_URL") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => DEFAULT_ELEVENLABS_CONVERSATION_WS_URL.to_string(),
    }
}

fn resolve_elevenlabs_conversation_timeout_secs_from_env() -> Result<u64, EstimateServiceError> {
    let value = match env::var("ELEVENLABS_CONVERSATION_TIMEOUT_SECS") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => return Ok(DEFAULT_ELEVENLABS_CONVERSATION_TIMEOUT_SECS),
    };

    let parsed = value
        .parse::<u64>()
        .map_err(|_| EstimateServiceError::InvalidConfiguration {
            key: "ELEVENLABS_CONVERSATION_TIMEOUT_SECS",
            message: "must be a positive integer".to_string(),
        })?;

    if parsed == 0 {
        return Err(EstimateServiceError::InvalidConfiguration {
            key: "ELEVENLABS_CONVERSATION_TIMEOUT_SECS",
            message: "must be greater than 0".to_string(),
        });
    }

    Ok(parsed)
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

fn resolve_elevenlabs_simulated_user_language_from_env() -> String {
    match env::var("ELEVENLABS_SIMULATED_USER_LANGUAGE") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => DEFAULT_ELEVENLABS_SIMULATED_USER_LANGUAGE.to_string(),
    }
}

fn resolve_elevenlabs_simulated_user_prompt_from_env() -> Option<String> {
    match env::var("ELEVENLABS_SIMULATED_USER_PROMPT") {
        Ok(value) if !value.trim().is_empty() => Some(value),
        _ => None,
    }
}

fn resolve_elevenlabs_simulated_user_llm_from_env() -> Option<String> {
    match env::var("ELEVENLABS_SIMULATED_USER_LLM") {
        Ok(value) if !value.trim().is_empty() => Some(value),
        _ => None,
    }
}

fn resolve_elevenlabs_use_backend_prompt_from_env() -> Result<bool, EstimateServiceError> {
    let value = match env::var("ELEVENLABS_USE_BACKEND_PROMPT") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => return Ok(false),
    };

    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(EstimateServiceError::InvalidConfiguration {
            key: "ELEVENLABS_USE_BACKEND_PROMPT",
            message: "must be a boolean (true/false)".to_string(),
        }),
    }
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
        extract_estimate_json_from_raw, resolve_elevenlabs_base_url_from_env,
        resolve_elevenlabs_conversation_timeout_secs_from_env,
        resolve_elevenlabs_conversation_ws_url_from_env,
        resolve_elevenlabs_new_turns_limit_from_env,
        resolve_elevenlabs_simulated_user_llm_from_env,
        resolve_elevenlabs_simulated_user_prompt_from_env,
        resolve_elevenlabs_use_backend_prompt_from_env, resolve_estimate_provider_from_env,
        resolve_openai_model_from_env,
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
            conversation_ws_url: "wss://api.elevenlabs.io/v1/convai/conversation".to_string(),
            conversation_timeout_secs: 10,
            new_turns_limit: 7,
            simulated_user_language: "en".to_string(),
            simulated_user_prompt: Some("Act as user".to_string()),
            simulated_user_llm: Some("gpt-4.1-mini".to_string()),
            use_backend_prompt: true,
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
    fn resolves_default_elevenlabs_conversation_ws_url_when_env_missing() {
        unsafe {
            std::env::remove_var("ELEVENLABS_CONVERSATION_WS_URL");
        }

        assert_eq!(
            resolve_elevenlabs_conversation_ws_url_from_env(),
            "wss://api.elevenlabs.io/v1/convai/conversation"
        );
    }

    #[test]
    #[serial]
    fn resolves_default_elevenlabs_conversation_timeout_when_env_missing() {
        unsafe {
            std::env::remove_var("ELEVENLABS_CONVERSATION_TIMEOUT_SECS");
        }

        assert!(matches!(
            resolve_elevenlabs_conversation_timeout_secs_from_env(),
            Ok(600)
        ));
    }

    #[test]
    #[serial]
    fn rejects_zero_elevenlabs_conversation_timeout() {
        unsafe {
            std::env::set_var("ELEVENLABS_CONVERSATION_TIMEOUT_SECS", "0");
        }

        assert!(matches!(
            resolve_elevenlabs_conversation_timeout_secs_from_env(),
            Err(EstimateServiceError::InvalidConfiguration {
                key: "ELEVENLABS_CONVERSATION_TIMEOUT_SECS",
                ..
            })
        ));
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
    fn resolves_default_simulated_user_prompt_when_env_missing() {
        unsafe {
            std::env::remove_var("ELEVENLABS_SIMULATED_USER_PROMPT");
        }

        assert_eq!(resolve_elevenlabs_simulated_user_prompt_from_env(), None);
    }

    #[test]
    #[serial]
    fn resolves_simulated_user_llm_from_env_when_present() {
        unsafe {
            std::env::set_var("ELEVENLABS_SIMULATED_USER_LLM", "gpt-4.1-mini");
        }

        assert_eq!(
            resolve_elevenlabs_simulated_user_llm_from_env().as_deref(),
            Some("gpt-4.1-mini")
        );
    }

    #[test]
    #[serial]
    fn rejects_invalid_use_backend_prompt_flag() {
        unsafe {
            std::env::set_var("ELEVENLABS_USE_BACKEND_PROMPT", "maybe");
        }

        let result = resolve_elevenlabs_use_backend_prompt_from_env();
        assert!(matches!(
            result,
            Err(EstimateServiceError::InvalidConfiguration {
                key: "ELEVENLABS_USE_BACKEND_PROMPT",
                ..
            })
        ));
    }

    #[test]
    #[serial]
    fn resolves_default_use_backend_prompt_to_false() {
        unsafe {
            std::env::remove_var("ELEVENLABS_USE_BACKEND_PROMPT");
        }

        assert!(matches!(
            resolve_elevenlabs_use_backend_prompt_from_env(),
            Ok(false)
        ));
    }

    #[test]
    fn extracts_estimate_json_from_markdown_fence() {
        let raw = "```json\n{\"tasks\":[{\"title\":\"X\"}]}\n```";
        let parsed = extract_estimate_json_from_raw(raw);
        assert_eq!(parsed.as_deref(), Some("{\"tasks\":[{\"title\":\"X\"}]}"));
    }

    #[test]
    fn extracts_estimate_json_embedded_in_plain_text() {
        let raw = "Result below: {\"tasks\":[{\"title\":\"X\"}],\"rationale\":\"ok\"}";
        let parsed = extract_estimate_json_from_raw(raw);
        assert_eq!(
            parsed.as_deref(),
            Some("{\"tasks\":[{\"title\":\"X\"}],\"rationale\":\"ok\"}")
        );
    }

    #[test]
    fn rejects_non_estimate_text_without_tasks_array() {
        let raw = "[warmly] Hello! I'm Sol, your software task estimator.";
        assert_eq!(extract_estimate_json_from_raw(raw), None);
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
                            "{\"tasks\":[{\"title\":\"Backend\",\"description\":\"Implementacja\",\"price_sol\":380,\"complexity\":3,\"rationale\":\"ok\"}],\"rationale\":\"ok\"}".to_string(),
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
            "{\"tasks\":[{\"title\":\"Backend\",\"description\":\"Implementacja\",\"price_sol\":380,\"complexity\":3,\"rationale\":\"ok\"}],\"rationale\":\"ok\"}"
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
