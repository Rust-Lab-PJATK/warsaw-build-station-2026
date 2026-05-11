use std::sync::{Arc, OnceLock, RwLock};

use axum::{
    Extension, Json,
    body::Bytes,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use loco_rs::prelude::*;
use serde::Serialize;
use serde_json::Value;

use crate::{
    data::job::Job,
    services::{
        estimate::{
            EstimateServiceError, EstimateTextService, EstimateTextServiceFactory,
            build_estimate_text_service_from_env,
        },
        estimate_flow::{EstimateFlowError, estimate_task},
        estimate_output::ValidatedEstimate,
        estimate_rag::{
            EstimateRagSyncError, EstimateRagSyncService, EstimateRagSyncServiceFactory,
            build_estimate_rag_sync_service_from_env,
        },
        job::JobService,
    },
    views::estimate::{
        EstimateRequest, EstimateResponse, EstimateServiceErrorResponse,
        EstimateValidationErrorResponse,
    },
};
use tracing::warn;

const ESTIMATE_PROVIDER_CONFIGURATION_ERROR_CODE: &str = "estimate_provider_configuration_error";
const ESTIMATE_PROVIDER_CONFIGURATION_ERROR_MESSAGE: &str =
    "estimate provider configuration is missing or invalid";
const ESTIMATE_PROVIDER_REQUEST_FAILED_CODE: &str = "estimate_provider_request_failed";
const ESTIMATE_PROVIDER_REQUEST_FAILED_MESSAGE: &str =
    "failed to get estimate from upstream provider";

static ESTIMATE_SERVICE_FACTORY_OVERRIDE: OnceLock<
    RwLock<Option<Arc<EstimateTextServiceFactory>>>,
> = OnceLock::new();
static ESTIMATE_RAG_SYNC_FACTORY_OVERRIDE: OnceLock<
    RwLock<Option<Arc<EstimateRagSyncServiceFactory>>>,
> = OnceLock::new();

#[debug_handler]
async fn estimate(
    Extension(job_service): Extension<Arc<JobService>>,
    body: Bytes,
) -> Result<Response> {
    let request = match parse_request(&body) {
        Ok(request) => request,
        Err(validation_error) => {
            return Ok(json_response(StatusCode::BAD_REQUEST, validation_error));
        }
    };

    if let Some(validation_error) = request.validate() {
        return Ok(json_response(StatusCode::BAD_REQUEST, validation_error));
    }

    let llm_service = match build_estimate_text_service() {
        Ok(service) => service,
        Err(error) => {
            return Ok(map_service_error(error));
        }
    };

    match estimate_task(&request.task_description, llm_service.as_ref()).await {
        Ok(estimate) => {
            sync_estimate_to_rag_best_effort(&request.task_description, &estimate).await;

            let job = Job::from(&estimate);
            if let Err(e) = job_service.save(&job).await {
                tracing::error!("Failed to save job: {}", e);
            }

            Ok(json_response(
                StatusCode::OK,
                EstimateResponse::from(estimate),
            ))
        }
        Err(error) => Ok(map_flow_error(error)),
    }
}

fn parse_request(body: &[u8]) -> Result<EstimateRequest, EstimateValidationErrorResponse> {
    let raw_value = serde_json::from_slice::<Value>(body)
        .map_err(|_| EstimateValidationErrorResponse::for_invalid_json_body())?;

    let object = raw_value
        .as_object()
        .ok_or_else(EstimateValidationErrorResponse::for_non_object_body)?;

    let raw_task_description = object
        .get("task_description")
        .ok_or_else(EstimateValidationErrorResponse::for_missing_task_description)?;

    let task_description = raw_task_description
        .as_str()
        .ok_or_else(EstimateValidationErrorResponse::for_invalid_task_description_type)?;

    Ok(EstimateRequest {
        task_description: task_description.to_string(),
    })
}

fn estimate_service_factory_override() -> &'static RwLock<Option<Arc<EstimateTextServiceFactory>>> {
    ESTIMATE_SERVICE_FACTORY_OVERRIDE.get_or_init(|| RwLock::new(None))
}

fn build_estimate_text_service()
-> std::result::Result<Box<dyn EstimateTextService>, EstimateServiceError> {
    let override_factory = {
        let lock = estimate_service_factory_override();
        let guard = match lock.read() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        guard.clone()
    };

    if let Some(factory) = override_factory {
        return factory();
    }

    build_estimate_text_service_from_env()
}

fn estimate_rag_sync_factory_override()
-> &'static RwLock<Option<Arc<EstimateRagSyncServiceFactory>>> {
    ESTIMATE_RAG_SYNC_FACTORY_OVERRIDE.get_or_init(|| RwLock::new(None))
}

fn build_estimate_rag_sync_service()
-> std::result::Result<Box<dyn EstimateRagSyncService>, EstimateRagSyncError> {
    let override_factory = {
        let lock = estimate_rag_sync_factory_override();
        let guard = match lock.read() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        guard.clone()
    };

    if let Some(factory) = override_factory {
        return factory();
    }

    build_estimate_rag_sync_service_from_env()
}

async fn sync_estimate_to_rag_best_effort(task_description: &str, estimate: &ValidatedEstimate) {
    let rag_service = match build_estimate_rag_sync_service() {
        Ok(service) => service,
        Err(error) => {
            warn!(
                error = %error,
                "estimate RAG sync skipped due to configuration/initialization error"
            );
            return;
        }
    };

    if let Err(error) = rag_service
        .sync_estimate_tasks(task_description, estimate)
        .await
    {
        warn!(error = %error, "estimate RAG sync failed");
    }
}

#[doc(hidden)]
pub fn set_estimate_service_factory_for_tests<F>(factory: F)
where
    F: Fn() -> std::result::Result<Box<dyn EstimateTextService>, EstimateServiceError>
        + Send
        + Sync
        + 'static,
{
    let lock = estimate_service_factory_override();
    let mut guard = match lock.write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *guard = Some(Arc::new(factory));
}

#[doc(hidden)]
pub fn clear_estimate_service_factory_for_tests() {
    let lock = estimate_service_factory_override();
    let mut guard = match lock.write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *guard = None;
}

#[doc(hidden)]
pub fn set_estimate_rag_sync_service_factory_for_tests<F>(factory: F)
where
    F: Fn() -> std::result::Result<Box<dyn EstimateRagSyncService>, EstimateRagSyncError>
        + Send
        + Sync
        + 'static,
{
    let lock = estimate_rag_sync_factory_override();
    let mut guard = match lock.write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *guard = Some(Arc::new(factory));
}

#[doc(hidden)]
pub fn clear_estimate_rag_sync_service_factory_for_tests() {
    let lock = estimate_rag_sync_factory_override();
    let mut guard = match lock.write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *guard = None;
}

fn json_response<T: Serialize>(status: StatusCode, payload: T) -> Response {
    (status, Json(payload)).into_response()
}

fn map_service_error(error: EstimateServiceError) -> Response {
    match error {
        EstimateServiceError::InvalidTaskDescription => json_response(
            StatusCode::BAD_REQUEST,
            EstimateValidationErrorResponse::for_blank_task_description(),
        ),
        EstimateServiceError::ClientInitialization(_) => json_response(
            StatusCode::SERVICE_UNAVAILABLE,
            EstimateServiceErrorResponse::new(
                ESTIMATE_PROVIDER_CONFIGURATION_ERROR_CODE,
                ESTIMATE_PROVIDER_CONFIGURATION_ERROR_MESSAGE,
            ),
        ),
        EstimateServiceError::LlmRequest(_) => json_response(
            StatusCode::BAD_GATEWAY,
            EstimateServiceErrorResponse::new(
                ESTIMATE_PROVIDER_REQUEST_FAILED_CODE,
                ESTIMATE_PROVIDER_REQUEST_FAILED_MESSAGE,
            ),
        ),
        EstimateServiceError::MissingConfiguration { .. }
        | EstimateServiceError::InvalidConfiguration { .. }
        | EstimateServiceError::ElevenLabsClientInitialization(_) => json_response(
            StatusCode::SERVICE_UNAVAILABLE,
            EstimateServiceErrorResponse::new(
                ESTIMATE_PROVIDER_CONFIGURATION_ERROR_CODE,
                ESTIMATE_PROVIDER_CONFIGURATION_ERROR_MESSAGE,
            ),
        ),
        EstimateServiceError::ElevenLabsRequestFailed(_)
        | EstimateServiceError::ElevenLabsRequestUnsuccessful { .. }
        | EstimateServiceError::ElevenLabsEmptyResponse => json_response(
            StatusCode::BAD_GATEWAY,
            EstimateServiceErrorResponse::new(
                ESTIMATE_PROVIDER_REQUEST_FAILED_CODE,
                ESTIMATE_PROVIDER_REQUEST_FAILED_MESSAGE,
            ),
        ),
    }
}

fn map_flow_error(error: EstimateFlowError) -> Response {
    match error {
        EstimateFlowError::InvalidTaskDescription => json_response(
            StatusCode::BAD_REQUEST,
            EstimateValidationErrorResponse::for_blank_task_description(),
        ),
        EstimateFlowError::Service(service_error) => map_service_error(service_error),
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/estimate", post(estimate))
}

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, http::StatusCode};
    use rig::{client::ProviderClientError, completion::CompletionError};

    use super::map_service_error;
    use super::parse_request;
    use crate::{
        services::estimate::EstimateServiceError,
        views::estimate::{EstimateServiceErrorResponse, EstimateValidationErrorResponse},
    };

    async fn assert_service_error_mapping(
        error: EstimateServiceError,
        expected_status: StatusCode,
        expected_code: &str,
        expected_message: &str,
    ) {
        let response = map_service_error(error);
        assert_eq!(response.status(), expected_status);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        let payload = serde_json::from_slice::<EstimateServiceErrorResponse>(&body)
            .expect("body should be valid EstimateServiceErrorResponse JSON");
        assert_eq!(
            payload,
            EstimateServiceErrorResponse::new(expected_code, expected_message)
        );
    }

    #[test]
    fn parse_request_rejects_invalid_json() {
        let result = parse_request(br#"{"task_description":"abc""#);
        assert_eq!(
            result.err(),
            Some(EstimateValidationErrorResponse::for_invalid_json_body())
        );
    }

    #[test]
    fn parse_request_rejects_non_object_json() {
        let result = parse_request(br#"[]"#);
        assert_eq!(
            result.err(),
            Some(EstimateValidationErrorResponse::for_non_object_body())
        );
    }

    #[test]
    fn parse_request_rejects_missing_task_description() {
        let result = parse_request(br#"{"description":"abc"}"#);
        assert_eq!(
            result.err(),
            Some(EstimateValidationErrorResponse::for_missing_task_description())
        );
    }

    #[test]
    fn parse_request_rejects_task_description_with_invalid_type() {
        let result = parse_request(br#"{"task_description": 123}"#);
        assert_eq!(
            result.err(),
            Some(EstimateValidationErrorResponse::for_invalid_task_description_type())
        );
    }

    #[test]
    fn parse_request_accepts_valid_json() {
        let result = parse_request(br#"{"task_description":"abc"}"#);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn map_service_error_openai_configuration_is_provider_neutral() {
        assert_service_error_mapping(
            EstimateServiceError::ClientInitialization(ProviderClientError::InvalidConfiguration(
                "invalid OPENAI_API_KEY",
            )),
            StatusCode::SERVICE_UNAVAILABLE,
            "estimate_provider_configuration_error",
            "estimate provider configuration is missing or invalid",
        )
        .await;
    }

    #[tokio::test]
    async fn map_service_error_openai_request_failure_is_provider_neutral() {
        assert_service_error_mapping(
            EstimateServiceError::LlmRequest(
                CompletionError::ProviderError("upstream failed".to_string()).into(),
            ),
            StatusCode::BAD_GATEWAY,
            "estimate_provider_request_failed",
            "failed to get estimate from upstream provider",
        )
        .await;
    }

    #[tokio::test]
    async fn map_service_error_elevenlabs_configuration_is_provider_neutral() {
        assert_service_error_mapping(
            EstimateServiceError::MissingConfiguration {
                key: "ELEVENLABS_API_KEY",
            },
            StatusCode::SERVICE_UNAVAILABLE,
            "estimate_provider_configuration_error",
            "estimate provider configuration is missing or invalid",
        )
        .await;
    }

    #[tokio::test]
    async fn map_service_error_elevenlabs_request_failure_is_provider_neutral() {
        assert_service_error_mapping(
            EstimateServiceError::ElevenLabsRequestUnsuccessful {
                status_code: 500,
                body: "upstream failed".to_string(),
            },
            StatusCode::BAD_GATEWAY,
            "estimate_provider_request_failed",
            "failed to get estimate from upstream provider",
        )
        .await;
    }
}
