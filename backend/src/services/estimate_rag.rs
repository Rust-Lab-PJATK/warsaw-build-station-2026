use std::{env, error::Error, fmt};

use async_trait::async_trait;
use reqwest::Client;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::services::estimate_output::{ValidatedEstimate, ValidatedTaskEstimate};

const DEFAULT_ELEVENLABS_BASE_URL: &str = "https://api.elevenlabs.io";
const DEFAULT_RAG_UPSERT_PATH: &str = "/v1/convai/knowledge-base/text";

#[async_trait]
pub trait EstimateRagSyncService: Send + Sync {
    async fn sync_estimate_tasks(
        &self,
        task_description: &str,
        estimate: &ValidatedEstimate,
    ) -> Result<(), EstimateRagSyncError>;
}

pub type EstimateRagSyncServiceFactory =
    dyn Fn() -> Result<Box<dyn EstimateRagSyncService>, EstimateRagSyncError> + Send + Sync;

pub fn build_estimate_rag_sync_service_from_env()
-> Result<Box<dyn EstimateRagSyncService>, EstimateRagSyncError> {
    ReqwestEstimateRagSyncService::from_env()
        .map(|service| Box::new(service) as Box<dyn EstimateRagSyncService>)
}

#[derive(Debug, Clone)]
pub struct EstimateRagConfig {
    api_key: String,
    agent_id: String,
    upsert_url: String,
    parent_folder_id: Option<String>,
}

impl EstimateRagConfig {
    pub fn from_env() -> Result<Self, EstimateRagSyncError> {
        let api_key = required_non_empty_env("ELEVENLABS_API_KEY")?;
        let agent_id = required_non_empty_env("ELEVENLABS_AGENT_ID")?;
        let upsert_url = resolve_rag_upsert_url_from_env()?;
        let parent_folder_id = env::var("ELEVENLABS_RAG_PARENT_FOLDER_ID")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        Ok(Self {
            api_key,
            agent_id,
            upsert_url,
            parent_folder_id,
        })
    }

    #[must_use]
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    #[must_use]
    pub fn upsert_url(&self) -> &str {
        &self.upsert_url
    }

    #[must_use]
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }

    #[must_use]
    pub fn parent_folder_id(&self) -> Option<&str> {
        self.parent_folder_id.as_deref()
    }
}

#[derive(Debug, Clone)]
pub struct ReqwestEstimateRagSyncService {
    client: Client,
    config: EstimateRagConfig,
}

impl ReqwestEstimateRagSyncService {
    pub fn from_env() -> Result<Self, EstimateRagSyncError> {
        let config = EstimateRagConfig::from_env()?;
        let client = Client::builder()
            .build()
            .map_err(EstimateRagSyncError::ClientInitialization)?;

        Ok(Self { client, config })
    }

    #[must_use]
    pub fn new(client: Client, config: EstimateRagConfig) -> Self {
        Self { client, config }
    }
}

#[async_trait]
impl EstimateRagSyncService for ReqwestEstimateRagSyncService {
    async fn sync_estimate_tasks(
        &self,
        task_description: &str,
        estimate: &ValidatedEstimate,
    ) -> Result<(), EstimateRagSyncError> {
        if estimate.tasks.is_empty() {
            return Err(EstimateRagSyncError::InvalidConfiguration {
                key: "estimate.tasks",
                message: "cannot sync empty task list".to_string(),
            });
        }

        for (index, task) in estimate.tasks.iter().enumerate() {
            let payload = build_rag_document_request(
                index,
                task_description,
                estimate,
                task,
                self.config.parent_folder_id(),
            );
            let response = self
                .client
                .post(self.config.upsert_url())
                .header("xi-api-key", self.config.api_key())
                .query(&[("agent_id", self.config.agent_id())])
                .json(&payload)
                .send()
                .await
                .map_err(EstimateRagSyncError::RequestFailed)?;

            let status = response.status();
            if !status.is_success() {
                let body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "failed to read ElevenLabs RAG error body".to_string());
                return Err(EstimateRagSyncError::RequestUnsuccessful {
                    status_code: status.as_u16(),
                    body,
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct ElevenLabsRagTextRequest {
    text: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_folder_id: Option<String>,
}

fn build_rag_document_request(
    task_index: usize,
    task_description: &str,
    estimate: &ValidatedEstimate,
    task: &ValidatedTaskEstimate,
    parent_folder_id: Option<&str>,
) -> ElevenLabsRagTextRequest {
    let document_id = build_document_id(task_index, task_description, task);
    let text = format!(
        "source: api_estimate\n\
         document_id: {}\n\
         task_index: {}\n\
         project_description: {}\n\
         task_title: {}\n\
         task_description: {}\n\
         price_sol: {:.2}\n\
         complexity: {}\n\
         task_rationale: {}\n\
         project_total_price_sol: {:.2}\n\
         project_overall_complexity: {}\n\
         project_rationale: {}",
        document_id,
        task_index,
        task_description.trim(),
        task.title,
        task.description,
        task.price_sol,
        task.complexity,
        task.rationale,
        estimate.total_price_sol,
        estimate.overall_complexity,
        estimate.rationale
    );

    ElevenLabsRagTextRequest {
        text,
        name: format!("estimate-task-{task_index}-{}", task.title),
        parent_folder_id: parent_folder_id.map(ToOwned::to_owned),
    }
}

fn build_document_id(
    task_index: usize,
    task_description: &str,
    task: &ValidatedTaskEstimate,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(task_description.trim().as_bytes());
    hasher.update(task_index.to_string().as_bytes());
    hasher.update(task.title.trim().as_bytes());
    hasher.update(task.description.trim().as_bytes());
    hasher.update(format!("{:.2}", task.price_sol).as_bytes());
    hasher.update(task.complexity.to_string().as_bytes());

    let hash = hasher.finalize();
    let hash_hex = hash
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("estimate-task-{hash_hex}")
}

fn required_non_empty_env(key: &'static str) -> Result<String, EstimateRagSyncError> {
    match env::var(key) {
        Ok(value) if !value.trim().is_empty() => Ok(value),
        _ => Err(EstimateRagSyncError::MissingConfiguration { key }),
    }
}

fn resolve_rag_upsert_url_from_env() -> Result<String, EstimateRagSyncError> {
    if let Ok(value) = env::var("ELEVENLABS_RAG_UPSERT_URL")
        && !value.trim().is_empty()
    {
        return Ok(value);
    }

    let base_url = match env::var("ELEVENLABS_BASE_URL") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => DEFAULT_ELEVENLABS_BASE_URL.to_string(),
    };

    Ok(format!(
        "{}{}",
        base_url.trim_end_matches('/'),
        DEFAULT_RAG_UPSERT_PATH
    ))
}

#[derive(Debug)]
pub enum EstimateRagSyncError {
    MissingConfiguration { key: &'static str },
    InvalidConfiguration { key: &'static str, message: String },
    ClientInitialization(reqwest::Error),
    RequestFailed(reqwest::Error),
    RequestUnsuccessful { status_code: u16, body: String },
}

impl fmt::Display for EstimateRagSyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingConfiguration { key } => {
                write!(f, "missing required RAG configuration key: {key}")
            }
            Self::InvalidConfiguration { key, message } => {
                write!(f, "invalid RAG configuration for {key}: {message}")
            }
            Self::ClientInitialization(_) => write!(
                f,
                "failed to initialize ElevenLabs RAG client from environment configuration"
            ),
            Self::RequestFailed(_) => write!(f, "failed to write estimate tasks to ElevenLabs RAG"),
            Self::RequestUnsuccessful { status_code, .. } => write!(
                f,
                "ElevenLabs RAG API responded with non-success status: {status_code}"
            ),
        }
    }
}

impl Error for EstimateRagSyncError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingConfiguration { .. } | Self::InvalidConfiguration { .. } => None,
            Self::ClientInitialization(source) | Self::RequestFailed(source) => Some(source),
            Self::RequestUnsuccessful { .. } => None,
        }
    }
}
