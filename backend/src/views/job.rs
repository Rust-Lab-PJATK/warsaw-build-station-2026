use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct JobTaskRequest {
    pub title: String,
    pub description: String,
    pub price_sol: f64,
    pub complexity: u8,
    pub rationale: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[allow(clippy::module_name_repetitions)]
pub struct JobRequest {
    pub tasks: Vec<JobTaskRequest>,
    pub total_price_sol: f64,
    pub overall_complexity: u8,
    pub rationale: String,
}

impl JobRequest {
    #[must_use]
    pub fn validate(&self) -> Option<JobValidationErrorResponse> {
        if self.tasks.is_empty() {
            return Some(JobValidationErrorResponse::for_empty_tasks());
        }

        if self.rationale.trim().is_empty() {
            return Some(JobValidationErrorResponse::for_empty_rationale());
        }

        None
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct JobTaskResponse {
    pub title: String,
    pub description: String,
    pub price_sol: f64,
    pub complexity: u8,
    pub rationale: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[allow(clippy::module_name_repetitions)]
pub struct JobResponse {
    pub id: String,
    pub tasks: Vec<JobTaskResponse>,
    pub total_price_sol: f64,
    pub overall_complexity: u8,
    pub rationale: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions)]
pub struct JobValidationErrorResponse {
    pub code: String,
    pub message: String,
    pub field_errors: BTreeMap<String, Vec<String>>,
}

impl JobValidationErrorResponse {
    const VALIDATION_ERROR_CODE: &'static str = "validation_error";

    #[must_use]
    pub fn for_empty_tasks() -> Self {
        let mut field_errors = BTreeMap::new();
        field_errors.insert("tasks".to_string(), vec!["must not be empty".to_string()]);

        Self {
            code: Self::VALIDATION_ERROR_CODE.to_string(),
            message: "Validation failed".to_string(),
            field_errors,
        }
    }

    #[must_use]
    pub fn for_empty_rationale() -> Self {
        let mut field_errors = BTreeMap::new();
        field_errors.insert(
            "rationale".to_string(),
            vec!["must not be blank".to_string()],
        );

        Self {
            code: Self::VALIDATION_ERROR_CODE.to_string(),
            message: "Validation failed".to_string(),
            field_errors,
        }
    }

    #[must_use]
    pub fn for_invalid_json_body() -> Self {
        Self {
            code: Self::VALIDATION_ERROR_CODE.to_string(),
            message: "request body must be valid JSON".to_string(),
            field_errors: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn for_non_object_body() -> Self {
        Self {
            code: Self::VALIDATION_ERROR_CODE.to_string(),
            message: "request body must be a JSON object".to_string(),
            field_errors: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions)]
pub struct JobErrorResponse {
    pub code: String,
    pub message: String,
}

impl JobErrorResponse {
    #[must_use]
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
        }
    }
}

// Backward compatibility aliases
pub type JobHistoryRequest = JobRequest;
pub type JobHistoryResponse = JobResponse;
pub type JobHistoryValidationErrorResponse = JobValidationErrorResponse;
pub type JobHistoryErrorResponse = JobErrorResponse;

#[cfg(test)]
mod tests {
    use super::JobRequest;

    #[test]
    fn request_validation_rejects_empty_tasks() {
        let request = JobRequest {
            tasks: vec![],
            total_price_sol: 100.0,
            overall_complexity: 3,
            rationale: "test".to_string(),
        };

        assert!(request.validate().is_some());
    }

    #[test]
    fn request_validation_rejects_empty_rationale() {
        let request = JobRequest {
            tasks: vec![],
            total_price_sol: 100.0,
            overall_complexity: 3,
            rationale: "  ".to_string(),
        };

        assert!(request.validate().is_some());
    }

    #[test]
    fn request_validation_allows_valid_request() {
        use super::JobTaskRequest;
        let request = JobRequest {
            tasks: vec![JobTaskRequest {
                title: "Task 1".to_string(),
                description: "Description".to_string(),
                price_sol: 100.0,
                complexity: 3,
                rationale: "task rationale".to_string(),
            }],
            total_price_sol: 100.0,
            overall_complexity: 3,
            rationale: "valid rationale".to_string(),
        };

        assert!(request.validate().is_none());
    }
}
