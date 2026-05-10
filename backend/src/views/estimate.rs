use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

const VALIDATION_ERROR_CODE: &str = "validation_error";

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions)]
pub struct EstimateRequest {
    pub task_description: String,
}

impl EstimateRequest {
    #[must_use]
    pub fn validate(&self) -> Option<EstimateValidationErrorResponse> {
        if self.task_description.trim().is_empty() {
            return Some(EstimateValidationErrorResponse::for_blank_task_description());
        }

        None
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[allow(clippy::module_name_repetitions)]
pub struct EstimateResponse {
    pub price_sol: f64,
    pub complexity: u8,
    pub rationale: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions)]
pub struct EstimateValidationErrorResponse {
    pub code: String,
    pub message: String,
    pub field_errors: BTreeMap<String, Vec<String>>,
}

impl EstimateValidationErrorResponse {
    #[must_use]
    pub fn for_blank_task_description() -> Self {
        let mut field_errors = BTreeMap::new();
        field_errors.insert(
            "task_description".to_string(),
            vec!["must not be blank".to_string()],
        );

        Self {
            code: VALIDATION_ERROR_CODE.to_string(),
            message: "Validation failed".to_string(),
            field_errors,
        }
    }

    #[must_use]
    pub fn for_invalid_json_body() -> Self {
        Self {
            code: VALIDATION_ERROR_CODE.to_string(),
            message: "request body must be valid JSON".to_string(),
            field_errors: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn for_non_object_body() -> Self {
        Self {
            code: VALIDATION_ERROR_CODE.to_string(),
            message: "request body must be a JSON object".to_string(),
            field_errors: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn for_missing_task_description() -> Self {
        let mut field_errors = BTreeMap::new();
        field_errors.insert(
            "task_description".to_string(),
            vec!["is required".to_string()],
        );

        Self {
            code: VALIDATION_ERROR_CODE.to_string(),
            message: "Validation failed".to_string(),
            field_errors,
        }
    }

    #[must_use]
    pub fn for_invalid_task_description_type() -> Self {
        let mut field_errors = BTreeMap::new();
        field_errors.insert(
            "task_description".to_string(),
            vec!["must be provided as a string".to_string()],
        );

        Self {
            code: VALIDATION_ERROR_CODE.to_string(),
            message: "Validation failed".to_string(),
            field_errors,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions)]
pub struct EstimateServiceErrorResponse {
    pub code: String,
    pub message: String,
}

impl EstimateServiceErrorResponse {
    #[must_use]
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
        }
    }
}

impl From<crate::services::estimate_output::ValidatedEstimate> for EstimateResponse {
    fn from(value: crate::services::estimate_output::ValidatedEstimate) -> Self {
        Self {
            price_sol: value.price_sol,
            complexity: value.complexity,
            rationale: value.rationale,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EstimateRequest;

    #[test]
    fn request_validation_rejects_blank_description() {
        let request = EstimateRequest {
            task_description: "  ".to_string(),
        };

        assert!(request.validate().is_some());
    }

    #[test]
    fn request_validation_allows_non_blank_description() {
        let request = EstimateRequest {
            task_description: "Build a booking app".to_string(),
        };

        assert!(request.validate().is_none());
    }
}
