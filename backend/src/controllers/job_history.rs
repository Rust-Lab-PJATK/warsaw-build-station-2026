use axum::{
    Json,
    body::Bytes,
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use loco_rs::prelude::*;
use serde::Serialize;
use serde_json::Value;

use crate::{
    data::job_history::JobHistoryEntry,
    data::job_history::JobTask,
    services::job_history::JobHistoryService,
    views::job_history::{
        JobHistoryErrorResponse, JobHistoryRequest, JobHistoryResponse,
        JobHistoryValidationErrorResponse, JobTaskResponse,
    },
};

const JOB_HISTORY_NOT_FOUND_ERROR_CODE: &str = "job_history_not_found";
const JOB_HISTORY_NOT_FOUND_ERROR_MESSAGE: &str = "Job history not found";
const JOB_HISTORY_DATABASE_ERROR_CODE: &str = "job_history_database_error";
const JOB_HISTORY_DATABASE_ERROR_MESSAGE: &str = "Database operation failed";

#[debug_handler]
async fn create_job_history(body: Bytes) -> Result<Response> {
    let request = match parse_request(&body) {
        Ok(request) => request,
        Err(validation_error) => {
            return Ok(json_response(StatusCode::BAD_REQUEST, validation_error));
        }
    };

    if let Some(validation_error) = request.validate() {
        return Ok(json_response(StatusCode::BAD_REQUEST, validation_error));
    }

    let service = match JobHistoryService::from_env().await {
        Ok(service) => service,
        Err(_) => {
            return Ok(json_response(
                StatusCode::SERVICE_UNAVAILABLE,
                JobHistoryErrorResponse::new(
                    JOB_HISTORY_DATABASE_ERROR_CODE,
                    "Failed to connect to database",
                ),
            ));
        }
    };

    let job_id = match service.get_next_job_id().await {
        Ok(id) => id,
        Err(_) => {
            return Ok(json_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                JobHistoryErrorResponse::new(
                    JOB_HISTORY_DATABASE_ERROR_CODE,
                    JOB_HISTORY_DATABASE_ERROR_MESSAGE,
                ),
            ));
        }
    };

    let tasks: Vec<JobTask> = request
        .tasks
        .into_iter()
        .map(|task| JobTask {
            title: task.title,
            description: task.description,
            price_sol: task.price_sol,
            complexity: task.complexity,
            rationale: task.rationale,
        })
        .collect();

    let entry = JobHistoryEntry::new(
        job_id,
        tasks,
        request.total_price_sol,
        request.overall_complexity,
        request.rationale,
    );

    match service.save(&entry).await {
        Ok(oid) => {
            let response = JobHistoryResponse {
                id: oid,
                job_id,
                tasks: entry
                    .tasks
                    .into_iter()
                    .map(|task| JobTaskResponse {
                        title: task.title,
                        description: task.description,
                        price_sol: task.price_sol,
                        complexity: task.complexity,
                        rationale: task.rationale,
                    })
                    .collect(),
                total_price_sol: entry.total_price_sol,
                overall_complexity: entry.overall_complexity,
                rationale: entry.rationale,
                created_at: entry.created_at,
                updated_at: entry.updated_at,
            };
            Ok(json_response(StatusCode::CREATED, response))
        }
        Err(_) => Ok(json_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            JobHistoryErrorResponse::new(
                JOB_HISTORY_DATABASE_ERROR_CODE,
                JOB_HISTORY_DATABASE_ERROR_MESSAGE,
            ),
        )),
    }
}

#[debug_handler]
async fn get_job_history(Path(job_id): Path<i64>) -> Result<Response> {
    let service = match JobHistoryService::from_env().await {
        Ok(service) => service,
        Err(_) => {
            return Ok(json_response(
                StatusCode::SERVICE_UNAVAILABLE,
                JobHistoryErrorResponse::new(
                    JOB_HISTORY_DATABASE_ERROR_CODE,
                    "Failed to connect to database",
                ),
            ));
        }
    };

    match service.get_by_job_id(job_id).await {
        Ok(Some(job_history)) => {
            let response = JobHistoryResponse {
                id: job_history.id.unwrap_or_default(),
                job_id: job_history.job_id,
                tasks: job_history
                    .tasks
                    .into_iter()
                    .map(|task| JobTaskResponse {
                        title: task.title,
                        description: task.description,
                        price_sol: task.price_sol,
                        complexity: task.complexity,
                        rationale: task.rationale,
                    })
                    .collect(),
                total_price_sol: job_history.total_price_sol,
                overall_complexity: job_history.overall_complexity,
                rationale: job_history.rationale,
                created_at: job_history.created_at,
                updated_at: job_history.updated_at,
            };
            Ok(json_response(StatusCode::OK, response))
        }
        Ok(None) => Ok(json_response(
            StatusCode::NOT_FOUND,
            JobHistoryErrorResponse::new(
                JOB_HISTORY_NOT_FOUND_ERROR_CODE,
                JOB_HISTORY_NOT_FOUND_ERROR_MESSAGE,
            ),
        )),
        Err(_) => Ok(json_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            JobHistoryErrorResponse::new(
                JOB_HISTORY_DATABASE_ERROR_CODE,
                JOB_HISTORY_DATABASE_ERROR_MESSAGE,
            ),
        )),
    }
}

#[debug_handler]
async fn list_entries() -> Result<Response> {
    let service = match JobHistoryService::from_env().await {
        Ok(service) => service,
        Err(_) => {
            return Ok(json_response(
                StatusCode::SERVICE_UNAVAILABLE,
                JobHistoryErrorResponse::new(
                    JOB_HISTORY_DATABASE_ERROR_CODE,
                    "Failed to connect to database",
                ),
            ));
        }
    };

    match service.get_all().await {
        Ok(entries) => {
            let responses: Vec<JobHistoryResponse> = entries
                .into_iter()
                .map(|entry| JobHistoryResponse {
                    id: entry.id.unwrap_or_default(),
                    job_id: entry.job_id,
                    tasks: entry
                        .tasks
                        .into_iter()
                        .map(|task| JobTaskResponse {
                            title: task.title,
                            description: task.description,
                            price_sol: task.price_sol,
                            complexity: task.complexity,
                            rationale: task.rationale,
                        })
                        .collect(),
                    total_price_sol: entry.total_price_sol,
                    overall_complexity: entry.overall_complexity,
                    rationale: entry.rationale,
                    created_at: entry.created_at,
                    updated_at: entry.updated_at,
                })
                .collect();

            Ok(json_response(
                StatusCode::OK,
                serde_json::json!({ "data": responses }),
            ))
        }
        Err(_) => Ok(json_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            JobHistoryErrorResponse::new(
                JOB_HISTORY_DATABASE_ERROR_CODE,
                JOB_HISTORY_DATABASE_ERROR_MESSAGE,
            ),
        )),
    }
}

fn parse_request(body: &[u8]) -> Result<JobHistoryRequest, JobHistoryValidationErrorResponse> {
    let raw_value = serde_json::from_slice::<Value>(body)
        .map_err(|_| JobHistoryValidationErrorResponse::for_invalid_json_body())?;

    let object = raw_value
        .as_object()
        .ok_or_else(JobHistoryValidationErrorResponse::for_non_object_body)?;

    serde_json::from_value(Value::Object(object.clone()))
        .map_err(|_| JobHistoryValidationErrorResponse::for_invalid_json_body())
}

fn json_response<T: Serialize>(status: StatusCode, payload: T) -> Response {
    (status, Json(payload)).into_response()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/job-history", post(create_job_history).get(list_entries))
        .add("/job-history/{job_id}", get(get_job_history))
}

#[cfg(test)]
mod tests {
    use super::parse_request;
    use crate::views::job_history::JobHistoryValidationErrorResponse;

    #[test]
    fn parse_request_rejects_invalid_json() {
        let result = parse_request(br#"{"tasks":["#);
        assert_eq!(
            result.err(),
            Some(JobHistoryValidationErrorResponse::for_invalid_json_body())
        );
    }

    #[test]
    fn parse_request_rejects_non_object_json() {
        let result = parse_request(br#"[]"#);
        assert_eq!(
            result.err(),
            Some(JobHistoryValidationErrorResponse::for_non_object_body())
        );
    }

    #[test]
    fn parse_request_accepts_valid_json() {
        let result = parse_request(
            br#"{
            "tasks": [{"title": "Test", "description": "Desc", "price_sol": 100.0, "complexity": 3, "rationale": "reason"}],
            "total_price_sol": 100.0,
            "overall_complexity": 3,
            "rationale": "Overall reason"
        }"#,
        );
        assert!(result.is_ok());
    }
}
