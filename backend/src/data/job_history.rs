use serde::{Deserialize, Serialize};

/// Task within a job history entry
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct JobTask {
    pub title: String,
    pub description: String,
    pub price_sol: f64,
    pub complexity: u8,
    pub rationale: String,
}

/// Complete job history document in MongoDB
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct JobHistoryEntry {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub job_id: i64,
    pub tasks: Vec<JobTask>,
    pub total_price_sol: f64,
    pub overall_complexity: u8,
    pub rationale: String,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

impl JobHistoryEntry {
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
            tasks,
            total_price_sol,
            overall_complexity,
            rationale,
            created_at: Some(now.clone()),
            updated_at: Some(now),
        }
    }
}
