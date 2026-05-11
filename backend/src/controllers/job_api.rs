use crate::data::job::JobStatus;
use crate::services::job::JobService;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Extension, Json};
use loco_rs::Error;
use loco_rs::prelude::Routes;
use std::sync::Arc;

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/task/{id}/preview/", post(preview))
}

pub(crate) async fn preview(
    Extension(job_service): Extension<Arc<JobService>>,
    Path(id): Path<String>,
) -> loco_rs::Result<Response> {
    let mut job = match job_service.get_by_id(&id).await {
        Ok(Some(job)) => job,
        Ok(None) => return Err(Error::NotFound),
        Err(e) => return Err(Error::Message(e.to_string())),
    };

    job.transition_to(JobStatus::AwaitingReview);

    if let Err(e) = job_service.update_by_id(&id, &job).await {
        return Err(Error::Message(format!(
            "Failed to update job {}: {}",
            id, e
        )));
    }

    Ok((StatusCode::OK, Json(job)).into_response())
}
