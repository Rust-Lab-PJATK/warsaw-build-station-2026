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
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use crate::{
    services::job::JobService,
    views::job::{
        JobErrorResponse, JobLinkTaskResponse, JobLinkTaskValidationErrorResponse,
    },
};

#[debug_handler]
async fn link_task(Path(id): Path<String>, body: Bytes) -> Result<Response> {
    let request = match parse_request(&body) {
        Ok(request) => request,
        Err(validation_error) => {
            return Ok(json_response(StatusCode::BAD_REQUEST, validation_error));
        }
    };

    let Some(task_pubkey) = request.task_pubkey else {
        return Ok(json_response(
            StatusCode::BAD_REQUEST,
            JobLinkTaskValidationErrorResponse::for_missing_task_pubkey(),
        ));
    };

    let task_pubkey = task_pubkey.trim();
    if task_pubkey.is_empty() {
        return Ok(json_response(
            StatusCode::BAD_REQUEST,
            JobLinkTaskValidationErrorResponse::for_blank_task_pubkey(),
        ));
    }

    if Pubkey::from_str(task_pubkey).is_err() {
        return Ok(json_response(
            StatusCode::BAD_REQUEST,
            JobLinkTaskValidationErrorResponse::for_invalid_task_pubkey(),
        ));
    }

    let job_service = JobService::from_env().await?;
    let Some(mut job) = job_service.get_by_id(&id).await? else {
        return Ok(json_response(
            StatusCode::NOT_FOUND,
            JobErrorResponse::new("job_not_found", "job not found"),
        ));
    };

    job.task_pubkey = Some(task_pubkey.to_string());
    job_service.update_by_id(&id, &job).await?;

    Ok(json_response(
        StatusCode::OK,
        JobLinkTaskResponse::new(id, task_pubkey.to_string()),
    ))
}

fn parse_request(body: &[u8])
-> Result<JobLinkTaskRequest, JobLinkTaskValidationErrorResponse> {
    let raw_value = serde_json::from_slice::<Value>(body)
        .map_err(|_| JobLinkTaskValidationErrorResponse::for_invalid_json_body())?;

    let object = raw_value
        .as_object()
        .ok_or_else(JobLinkTaskValidationErrorResponse::for_non_object_body)?;

    let task_pubkey = object.get("task_pubkey").and_then(|val| val.as_str());

    Ok(JobLinkTaskRequest {
        task_pubkey: task_pubkey.map(str::to_string),
    })
}

fn json_response<T: Serialize>(status: StatusCode, payload: T) -> Response {
    (status, Json(payload)).into_response()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct JobLinkTaskRequest {
    task_pubkey: Option<String>,
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/jobs")
        .add(":id/link-task", post(link_task))
}
