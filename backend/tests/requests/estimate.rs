use async_trait::async_trait;
use backend::{
    app::App,
    controllers::estimate::{
        clear_estimate_service_factory_for_tests, set_estimate_service_factory_for_tests,
    },
    services::{
        estimate::{EstimateServiceError, EstimateTextService},
        estimate_output::parse_and_validate_estimate_output,
    },
    views::estimate::EstimateValidationErrorResponse,
};
use loco_rs::testing::prelude::*;
use serial_test::serial;

struct StubEstimateTextService {
    output: String,
}

struct StubFailingEstimateTextService;

#[async_trait]
impl EstimateTextService for StubEstimateTextService {
    async fn generate_estimate_text(
        &self,
        _task_description: &str,
    ) -> std::result::Result<String, EstimateServiceError> {
        Ok(self.output.clone())
    }
}

#[async_trait]
impl EstimateTextService for StubFailingEstimateTextService {
    async fn generate_estimate_text(
        &self,
        _task_description: &str,
    ) -> std::result::Result<String, EstimateServiceError> {
        Err(EstimateServiceError::ElevenLabsRequestUnsuccessful {
            status_code: 502,
            body: "upstream failed".to_string(),
        })
    }
}

struct EstimateServiceFactoryGuard;

impl Drop for EstimateServiceFactoryGuard {
    fn drop(&mut self) {
        clear_estimate_service_factory_for_tests();
    }
}

fn install_stub_service(raw_output: &str) -> EstimateServiceFactoryGuard {
    let output = raw_output.to_string();
    set_estimate_service_factory_for_tests(move || {
        Ok(Box::new(StubEstimateTextService {
            output: output.clone(),
        }))
    });
    EstimateServiceFactoryGuard
}

fn install_stub_service_init_failure() -> EstimateServiceFactoryGuard {
    set_estimate_service_factory_for_tests(|| {
        Err(EstimateServiceError::MissingConfiguration {
            key: "ELEVENLABS_API_KEY",
        })
    });
    EstimateServiceFactoryGuard
}

fn install_stub_service_request_failure() -> EstimateServiceFactoryGuard {
    set_estimate_service_factory_for_tests(|| Ok(Box::new(StubFailingEstimateTextService)));
    EstimateServiceFactoryGuard
}

#[tokio::test]
#[serial]
async fn post_estimate_returns_success_payload_shape() {
    let _guard = install_stub_service(
        r#"{"price_sol": 380, "complexity": 3, "rationale": "Zakres średni."}"#,
    );

    request::<App, _, _>(|request, _ctx| async move {
        let res = request
            .post("/api/estimate")
            .json(&serde_json::json!({"task_description":"Dodaj endpoint API"}))
            .await;

        assert_eq!(res.status_code(), 200);
        res.assert_json(&serde_json::json!({
            "price_sol": 380.0,
            "complexity": 3,
            "rationale": "Zakres średni."
        }));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn post_estimate_uses_deterministic_fallback_when_model_output_is_malformed() {
    let malformed_output = "to nie jest json";
    let _guard = install_stub_service(malformed_output);
    let task_description = "Dodaj endpoint API";
    let expected_fallback = parse_and_validate_estimate_output(task_description, malformed_output);

    request::<App, _, _>(|request, _ctx| async move {
        let res = request
            .post("/api/estimate")
            .json(&serde_json::json!({ "task_description": task_description }))
            .await;

        assert_eq!(res.status_code(), 200);
        res.assert_json(&serde_json::json!({
            "price_sol": expected_fallback.estimate.price_sol,
            "complexity": expected_fallback.estimate.complexity,
            "rationale": expected_fallback.estimate.rationale
        }));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn post_estimate_rejects_invalid_request_body() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request
            .post("/api/estimate")
            .json(&serde_json::json!(["bad"]))
            .await;

        assert_eq!(res.status_code(), 400);
        res.assert_json(&serde_json::json!(
            EstimateValidationErrorResponse::for_non_object_body()
        ));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn post_estimate_rejects_blank_task_description() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request
            .post("/api/estimate")
            .json(&serde_json::json!({"task_description":"   "}))
            .await;

        assert_eq!(res.status_code(), 400);
        res.assert_json(&serde_json::json!(
            EstimateValidationErrorResponse::for_blank_task_description()
        ));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn post_estimate_maps_provider_configuration_errors_to_provider_neutral_payload() {
    let _guard = install_stub_service_init_failure();

    request::<App, _, _>(|request, _ctx| async move {
        let res = request
            .post("/api/estimate")
            .json(&serde_json::json!({"task_description":"Dodaj endpoint API"}))
            .await;

        assert_eq!(res.status_code(), 503);
        res.assert_json(&serde_json::json!({
            "code": "estimate_provider_configuration_error",
            "message": "estimate provider configuration is missing or invalid"
        }));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn post_estimate_maps_provider_request_errors_to_provider_neutral_payload() {
    let _guard = install_stub_service_request_failure();

    request::<App, _, _>(|request, _ctx| async move {
        let res = request
            .post("/api/estimate")
            .json(&serde_json::json!({"task_description":"Dodaj endpoint API"}))
            .await;

        assert_eq!(res.status_code(), 502);
        res.assert_json(&serde_json::json!({
            "code": "estimate_provider_request_failed",
            "message": "failed to get estimate from upstream provider"
        }));
    })
    .await;
}
