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

#[async_trait]
impl EstimateTextService for StubEstimateTextService {
    async fn generate_estimate_text(
        &self,
        _task_description: &str,
    ) -> std::result::Result<String, EstimateServiceError> {
        Ok(self.output.clone())
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

#[tokio::test]
#[serial]
async fn post_estimate_returns_success_payload_shape() {
    let _guard = install_stub_service(
        r#"{"price_usdc": 380, "complexity": 3, "rationale": "Zakres średni."}"#,
    );

    request::<App, _, _>(|request, _ctx| async move {
        let res = request
            .post("/api/estimate")
            .json(&serde_json::json!({"task_description":"Dodaj endpoint API"}))
            .await;

        assert_eq!(res.status_code(), 200);
        res.assert_json(&serde_json::json!({
            "price_usdc": 380.0,
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
    let expected_fallback =
        parse_and_validate_estimate_output(task_description, malformed_output);

    request::<App, _, _>(|request, _ctx| async move {
        let res = request
            .post("/api/estimate")
            .json(&serde_json::json!({ "task_description": task_description }))
            .await;

        assert_eq!(res.status_code(), 200);
        res.assert_json(&serde_json::json!({
            "price_usdc": expected_fallback.estimate.price_usdc,
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
