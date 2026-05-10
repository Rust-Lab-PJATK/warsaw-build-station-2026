use std::sync::{Arc, OnceLock, RwLock};

use axum::{
    Json,
    body::Bytes,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use loco_rs::prelude::*;
use serde::Serialize;
use serde_json::Value;

use crate::{
    services::{
        estimate::{EstimateServiceError, EstimateTextService, RigOpenAiEstimateService},
        estimate_flow::{EstimateFlowError, estimate_task},
    },
    views::estimate::{
        EstimateRequest, EstimateResponse, EstimateServiceErrorResponse,
        EstimateValidationErrorResponse,
    },
};

type EstimateServiceFactory =
    dyn Fn() -> std::result::Result<Box<dyn EstimateTextService>, EstimateServiceError>
        + Send
        + Sync;

static ESTIMATE_SERVICE_FACTORY_OVERRIDE: OnceLock<RwLock<Option<Arc<EstimateServiceFactory>>>> =
    OnceLock::new();

#[debug_handler]
async fn estimate(body: Bytes) -> Result<Response> {
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
        Ok(estimate) => Ok(json_response(
            StatusCode::OK,
            EstimateResponse::from(estimate),
        )),
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

fn estimate_service_factory_override() -> &'static RwLock<Option<Arc<EstimateServiceFactory>>> {
    ESTIMATE_SERVICE_FACTORY_OVERRIDE.get_or_init(|| RwLock::new(None))
}

fn build_estimate_text_service() -> std::result::Result<
    Box<dyn EstimateTextService>,
    EstimateServiceError,
> {
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

    RigOpenAiEstimateService::from_env()
        .map(|service| Box::new(service) as Box<dyn EstimateTextService>)
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
                "openai_configuration_error",
                "OpenAI configuration is missing or invalid",
            ),
        ),
        EstimateServiceError::LlmRequest(_) => json_response(
            StatusCode::BAD_GATEWAY,
            EstimateServiceErrorResponse::new(
                "openai_request_failed",
                "failed to get estimate from OpenAI model",
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
    use super::parse_request;
    use crate::views::estimate::EstimateValidationErrorResponse;

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
}
