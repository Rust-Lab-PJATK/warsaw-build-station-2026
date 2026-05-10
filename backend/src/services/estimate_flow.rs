use std::{error::Error, fmt};

use crate::services::{
    estimate::{EstimateServiceError, EstimateTextService},
    estimate_output::{ValidatedEstimate, parse_and_validate_estimate_output},
};

#[derive(Debug)]
pub enum EstimateFlowError {
    InvalidTaskDescription,
    Service(EstimateServiceError),
}

impl fmt::Display for EstimateFlowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTaskDescription => write!(f, "task description must not be blank"),
            Self::Service(error) => write!(f, "{error}"),
        }
    }
}

impl Error for EstimateFlowError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidTaskDescription => None,
            Self::Service(error) => Some(error),
        }
    }
}

pub async fn estimate_task(
    task_description: &str,
    text_service: &dyn EstimateTextService,
) -> Result<ValidatedEstimate, EstimateFlowError> {
    if task_description.trim().is_empty() {
        return Err(EstimateFlowError::InvalidTaskDescription);
    }

    let raw_output = text_service
        .generate_estimate_text(task_description)
        .await
        .map_err(EstimateFlowError::Service)?;

    let parsed_result = parse_and_validate_estimate_output(task_description, &raw_output);
    Ok(parsed_result.estimate)
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use super::estimate_task;
    use crate::services::estimate::{EstimateServiceError, EstimateTextService};

    struct StubEstimateTextService {
        response_text: String,
    }

    #[async_trait]
    impl EstimateTextService for StubEstimateTextService {
        async fn generate_estimate_text(
            &self,
            _task_description: &str,
        ) -> Result<String, EstimateServiceError> {
            Ok(self.response_text.clone())
        }
    }

    #[tokio::test]
    async fn estimate_task_returns_validated_estimate_when_model_output_is_valid() {
        let text_service = StubEstimateTextService {
            response_text: r#"{"price_sol": 380, "complexity": 3, "rationale": "Zakres średni."}"#
                .to_string(),
        };

        let result = estimate_task("Dodaj API endpoint", &text_service).await;
        assert!(result.is_ok());
        if let Ok(estimate) = result {
            assert_eq!(estimate.price_sol, 380.0);
            assert_eq!(estimate.complexity, 3);
            assert_eq!(estimate.rationale, "Zakres średni.");
        }
    }

    #[tokio::test]
    async fn estimate_task_uses_fallback_when_model_output_is_invalid() {
        let text_service = StubEstimateTextService {
            response_text: "to nie jest json".to_string(),
        };

        let result = estimate_task("Dodaj API endpoint", &text_service).await;
        assert!(result.is_ok());
        if let Ok(estimate) = result {
            assert!((1.0..=100_000.0).contains(&estimate.price_sol));
            assert!((1..=5).contains(&i32::from(estimate.complexity)));
            assert!(estimate.rationale.contains("Użyto fallbacku estymacji"));
        }
    }
}
