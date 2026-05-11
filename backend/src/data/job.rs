use crate::services::estimate_output::{ValidatedEstimate, ValidatedTaskEstimate};
use serde::{Deserialize, Serialize};

/// Job status state machine
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum JobStatus {
    /// Price estimation complete
    Priced,
    /// Deposit received (Deposit)
    Deposit,
    /// Advance payment released
    Advance,
    /// Work preview submitted (Preview)
    AwaitingReview,
    /// Work accepted and completed
    Accepted,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Priced => write!(f, "PRICED"),
            JobStatus::Deposit => write!(f, "DEPOSIT"),
            JobStatus::Advance => write!(f, "ADVANCE"),
            JobStatus::AwaitingReview => write!(f, "AWAITING_REVIEW"),
            JobStatus::Accepted => write!(f, "ACCEPTED"),
        }
    }
}

/// Task within a job
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct JobTask {
    pub title: String,
    pub description: String,
    pub price_sol: f64,
    pub complexity: u8,
    pub rationale: String,
}

impl JobTask {
    pub fn new(
        title: String,
        description: String,
        price_sol: f64,
        complexity: u8,
        rationale: String,
    ) -> Self {
        Self {
            title,
            description,
            price_sol,
            complexity,
            rationale,
        }
    }

    pub fn from(estimate_task: &ValidatedTaskEstimate) -> Self {
        Self::new(
            estimate_task.title.clone(),
            estimate_task.description.clone(),
            estimate_task.price_sol,
            estimate_task.complexity,
            estimate_task.rationale.clone(),
        )
    }
}

/// Complete job document in MongoDB (jobs collection)
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Job {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub status: JobStatus,
    pub tasks: Vec<JobTask>,
    pub total_price_sol: f64,
    pub overall_complexity: u8,
    pub rationale: String,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

impl Job {
    pub fn new(
        tasks: Vec<JobTask>,
        total_price_sol: f64,
        overall_complexity: u8,
        rationale: String,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: Some(mongodb::bson::oid::ObjectId::new().to_hex()),
            status: JobStatus::Priced,
            tasks,
            total_price_sol,
            overall_complexity,
            rationale,
            created_at: Some(now.clone()),
            updated_at: Some(now),
        }
    }

    pub fn from(estimate: &ValidatedEstimate) -> Self {
        let tasks = estimate.tasks.iter().map(JobTask::from).collect::<Vec<_>>();

        Self::new(
            tasks,
            estimate.total_price_sol,
            estimate.overall_complexity,
            estimate.rationale.clone(),
        )
    }

    /// Transition to next job status
    pub fn transition_to(&mut self, status: JobStatus) {
        self.status = status;
        self.updated_at = Some(chrono::Utc::now().to_rfc3339());
    }
}
