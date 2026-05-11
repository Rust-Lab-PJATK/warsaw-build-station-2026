pub const SYSTEM_PROMPT: &str = include_str!("../../assets/PROMPT.md");

#[derive(Debug, Clone, Copy)]
struct FewShotExample {
    description: &'static str,
    output_json: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct HistoricalTransaction {
    reference: &'static str,
    summary: &'static str,
    price_sol: &'static str,
    complexity: u8,
}

const FEW_SHOT_EXAMPLES: [FewShotExample; 5] = [
    FewShotExample {
        description: "Build an admin panel for user and role management.",
        output_json: r#"{"tasks":[{"title":"Role and permission model","description":"Implement role entities, permission rules, and authorization checks.","price_sol":260.0,"complexity":4,"rationale":"Changes impact critical access-control flows."},{"title":"Admin panel UI","description":"Create user list, role editing, and filtering screens.","price_sol":310.0,"complexity":3,"rationale":"Mostly frontend work with API integration."},{"title":"Security regression tests","description":"Add tests for authorization paths and edge cases.","price_sol":180.0,"complexity":3,"rationale":"Protects against regressions in sensitive areas."}],"rationale":"The project spans backend authorization and frontend administration views."}"#,
    },
    FewShotExample {
        description: "Integrate crypto payments with asynchronous status webhooks.",
        output_json: r#"{"tasks":[{"title":"Payment provider integration","description":"Connect provider API and create payment transaction flow.","price_sol":340.0,"complexity":4,"rationale":"Core external API integration with business impact."},{"title":"Webhook handling and idempotency","description":"Process status callbacks and prevent duplicate processing.","price_sol":280.0,"complexity":4,"rationale":"Requires robust async and reliability handling."},{"title":"Monitoring and alerting","description":"Add logs and alerts for failed payment events.","price_sol":140.0,"complexity":2,"rationale":"Operational hardening with lower implementation risk."}],"rationale":"Most complexity comes from resilient asynchronous event handling."}"#,
    },
    FewShotExample {
        description: "Migrate authentication from monolith to a dedicated microservice.",
        output_json: r#"{"tasks":[{"title":"Auth API contract design","description":"Define auth endpoints and token lifecycle contract.","price_sol":190.0,"complexity":3,"rationale":"Stable interfaces are required before migration."},{"title":"Auth microservice implementation","description":"Build service logic for login, token issue, and refresh.","price_sol":420.0,"complexity":5,"rationale":"Largest engineering scope and highest technical complexity."},{"title":"Migration and compatibility rollout","description":"Switch traffic gradually and preserve backward compatibility.","price_sol":240.0,"complexity":4,"rationale":"Rollout strategy reduces production regression risk."}],"rationale":"This is a high-risk integration project that needs phased delivery."}"#,
    },
    FewShotExample {
        description: "Add file upload with antivirus scan and secure preview.",
        output_json: r#"{"tasks":[{"title":"Upload and storage","description":"Implement upload pipeline and object storage integration.","price_sol":220.0,"complexity":3,"rationale":"Standard backend integration task."},{"title":"Antivirus scanning","description":"Integrate malware scanner and quarantine unsafe files.","price_sol":260.0,"complexity":4,"rationale":"Adds security-sensitive processing pipeline."},{"title":"Preview and access control","description":"Generate previews and enforce role-based access.","price_sol":210.0,"complexity":3,"rationale":"Combines UI behavior with security checks."}],"rationale":"Medium complexity project with clear separation into 3 tasks."}"#,
    },
    FewShotExample {
        description: "Fix CSV export encoding and delimiter issues.",
        output_json: r#"{"tasks":[{"title":"Export bug fix","description":"Correct delimiter and UTF-8 handling in CSV export.","price_sol":95.0,"complexity":1,"rationale":"Small, localized bug fix."},{"title":"Regression test","description":"Add coverage for export edge cases and encoding.","price_sol":60.0,"complexity":1,"rationale":"Low-effort safety net for future changes."}],"rationale":"Small scope and low risk with fast turnaround."}"#,
    },
];

const HISTORICAL_TRANSACTIONS: [HistoricalTransaction; 6] = [
    HistoricalTransaction {
        reference: "TX-2026-001",
        summary: "Added REST endpoint for order status retrieval.",
        price_sol: "210.0",
        complexity: 2,
    },
    HistoricalTransaction {
        reference: "TX-2026-007",
        summary: "Integrated Slack webhook notifications for incidents.",
        price_sol: "300.0",
        complexity: 2,
    },
    HistoricalTransaction {
        reference: "TX-2026-014",
        summary: "Migrated customer database with field mapping and data validation.",
        price_sol: "980.0",
        complexity: 5,
    },
    HistoricalTransaction {
        reference: "TX-2026-019",
        summary: "Added Redis cache for offer list with expiration strategy.",
        price_sol: "560.0",
        complexity: 3,
    },
    HistoricalTransaction {
        reference: "TX-2026-021",
        summary: "Implemented S3 upload flow with signed URLs.",
        price_sol: "640.0",
        complexity: 4,
    },
    HistoricalTransaction {
        reference: "TX-2026-027",
        summary: "Extended KPI report with new metrics and PDF export.",
        price_sol: "760.0",
        complexity: 4,
    },
];

pub fn build_estimation_prompt(task_description: &str) -> String {
    let normalized_description = task_description.trim();

    [
        "### SYSTEM INSTRUCTION".to_string(),
        SYSTEM_PROMPT.to_string(),
        String::new(),
        "### HISTORICAL CONTEXT (MVP)".to_string(),
        format_historical_transactions(),
        String::new(),
        "### FEW-SHOT EXAMPLES".to_string(),
        format_few_shot_examples(),
        String::new(),
        "### NEW PROJECT TO ESTIMATE".to_string(),
        format!("Description: {normalized_description}"),
        "Return JSON only, following the system instruction.".to_string(),
    ]
    .join("\n")
}

fn format_historical_transactions() -> String {
    HISTORICAL_TRANSACTIONS
        .iter()
        .map(|transaction| {
            format!(
                "- {} | summary: {} | price: {} SOL | complexity: {}",
                transaction.reference,
                transaction.summary,
                transaction.price_sol,
                transaction.complexity
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_few_shot_examples() -> String {
    FEW_SHOT_EXAMPLES
        .iter()
        .enumerate()
        .map(|(index, example)| {
            format!(
                "Example {}:\nInput: {}\nOutput: {}",
                index + 1,
                example.description,
                example.output_json
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::build_estimation_prompt;

    #[test]
    fn estimation_prompt_includes_required_sections() {
        let prompt = build_estimation_prompt("Add profile update endpoint.");

        assert!(prompt.contains("### SYSTEM INSTRUCTION"));
        assert!(prompt.contains("### HISTORICAL CONTEXT (MVP)"));
        assert!(prompt.contains("### FEW-SHOT EXAMPLES"));
        assert!(prompt.contains("### NEW PROJECT TO ESTIMATE"));
    }

    #[test]
    fn estimation_prompt_embeds_task_description() {
        let prompt = build_estimation_prompt("New VAT invoicing integration.");

        assert!(prompt.contains("Description: New VAT invoicing integration."));
        assert!(prompt.contains("\"tasks\""));
        assert!(prompt.contains("\"price_sol\""));
    }
}
