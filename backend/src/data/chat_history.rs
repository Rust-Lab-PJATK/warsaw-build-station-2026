use serde::{Deserialize, Serialize};

/// Chat history entry - stores chat prompts and resulting job creation
/// Each entry has a unique MongoDB ObjectId (`id`) as its identifier.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ChatHistoryEntry {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub wallet_address: String,
    pub prompt: String,
    pub job_id: String, // Changed from i64 to String (ObjectId)
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

impl ChatHistoryEntry {
    pub fn new(wallet_address: String, prompt: String, job_id: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: Some(mongodb::bson::oid::ObjectId::new().to_hex()),
            wallet_address,
            prompt,
            job_id,
            created_at: Some(now.clone()),
            updated_at: Some(now),
        }
    }
}
