use serde::{Deserialize, Serialize};

/// Job status state machine
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum JobStatus {
    /// Price estimation complete (Wyceniono)
    Priced,
    /// Deposit received (Deposit)
    Deposit,
    /// Advance payment released (Zaliczka)
    Advance,
    /// Work preview submitted (Preview)
    Preview,
    /// Work accepted and completed (Zaakceptowano)
    Accepted,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Priced => write!(f, "PRICED"),
            JobStatus::Deposit => write!(f, "DEPOSIT"),
            JobStatus::Advance => write!(f, "ADVANCE"),
            JobStatus::Preview => write!(f, "PREVIEW"),
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

/// Complete job document in MongoDB (jobs collection)
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Job {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub job_id: i64,
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
        job_id: i64,
        tasks: Vec<JobTask>,
        total_price_sol: f64,
        overall_complexity: u8,
        rationale: String,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: Some(mongodb::bson::oid::ObjectId::new().to_hex()),
            job_id,
            status: JobStatus::Priced,
            tasks,
            total_price_sol,
            overall_complexity,
            rationale,
            created_at: Some(now.clone()),
            updated_at: Some(now),
        }
    }

    /// Transition to next job status
    pub fn transition_to(&mut self, status: JobStatus) {
        self.status = status;
        self.updated_at = Some(chrono::Utc::now().to_rfc3339());
    }
}

// Backward compatibility alias
pub type JobHistoryEntry = Job;
