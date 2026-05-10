use serde::Serialize;
use serde_json::Value;

const MIN_PRICE_SOL: f64 = 1.0;
const MAX_PRICE_SOL: f64 = 100_000.0;
const PRICE_PRECISION_SCALE: f64 = 100.0;
const MIN_COMPLEXITY: i64 = 1;
const MAX_COMPLEXITY: i64 = 5;
const FALLBACK_BASE_PRICE_PER_COMPLEXITY: f64 = 220.0;
const FALLBACK_PRICE_PER_WORD: f64 = 6.0;
const FALLBACK_MAX_WORDS: usize = 80;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ValidatedTaskEstimate {
    pub title: String,
    pub description: String,
    pub price_sol: f64,
    pub complexity: u8,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ValidatedEstimate {
    pub tasks: Vec<ValidatedTaskEstimate>,
    pub total_price_sol: f64,
    pub overall_complexity: u8,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EstimateParseResult {
    pub estimate: ValidatedEstimate,
    pub used_fallback: bool,
}

/// Parses and validates raw LLM output.
///
/// If parsing or validation fails, this function returns a deterministic fallback estimate.
#[must_use]
pub fn parse_and_validate_estimate_output(
    task_description: &str,
    raw_model_output: &str,
) -> EstimateParseResult {
    let fallback_with_reason = |reason: String| EstimateParseResult {
        estimate: build_fallback_estimate(task_description, &reason),
        used_fallback: true,
    };

    let parsed_json = match parse_json_object(raw_model_output) {
        Ok(value) => value,
        Err(error) => return fallback_with_reason(error),
    };

    match validate_json_estimate(&parsed_json) {
        Ok(estimate) => EstimateParseResult {
            estimate,
            used_fallback: false,
        },
        Err(error) => fallback_with_reason(error),
    }
}

fn parse_json_object(raw_model_output: &str) -> Result<Value, String> {
    let trimmed_output = raw_model_output.trim();

    if trimmed_output.is_empty() {
        return Err("model output was blank".to_string());
    }

    if let Ok(value) = serde_json::from_str::<Value>(trimmed_output) {
        return ensure_json_object(value);
    }

    let mut parse_error: Option<String> = None;

    for json_slice in extract_json_object_slices(trimmed_output) {
        match serde_json::from_str::<Value>(json_slice) {
            Ok(value) => return ensure_json_object(value),
            Err(error) => {
                if parse_error.is_none() {
                    parse_error = Some(format!("model output JSON parsing failed: {error}"));
                }
            }
        }
    }

    match parse_error {
        Some(error) => Err(error),
        None => Err("model output did not contain a JSON object".to_string()),
    }
}

fn extract_json_object_slices(text: &str) -> Vec<&str> {
    let mut slices = Vec::new();
    let mut depth = 0usize;
    let mut start_index: Option<usize> = None;
    let mut in_string = false;
    let mut escaped = false;

    for (index, character) in text.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }

            match character {
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }

            continue;
        }

        match character {
            '"' => in_string = true,
            '{' => {
                if depth == 0 {
                    start_index = Some(index);
                }
                depth += 1;
            }
            '}' => {
                if depth == 0 {
                    continue;
                }

                depth -= 1;
                if depth == 0
                    && let Some(start) = start_index.take()
                    && let Some(slice) = text.get(start..=index)
                {
                    slices.push(slice);
                }
            }
            _ => {}
        }
    }

    slices
}

fn ensure_json_object(value: Value) -> Result<Value, String> {
    if value.is_object() {
        Ok(value)
    } else {
        Err("model output must be a JSON object".to_string())
    }
}

fn validate_json_estimate(value: &Value) -> Result<ValidatedEstimate, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "model output must be a JSON object".to_string())?;

    let tasks = parse_tasks_field(object.get("tasks"))?;
    let total_price_sol = round_two_decimals(tasks.iter().map(|task| task.price_sol).sum::<f64>());
    let overall_complexity = tasks.iter().map(|task| task.complexity).max().unwrap_or(1);
    let rationale = parse_optional_rationale_field(object.get("rationale"), tasks.len())?;

    Ok(ValidatedEstimate {
        tasks,
        total_price_sol,
        overall_complexity,
        rationale,
    })
}

fn parse_tasks_field(tasks_value: Option<&Value>) -> Result<Vec<ValidatedTaskEstimate>, String> {
    let raw_tasks = tasks_value.ok_or_else(|| "missing required field: tasks".to_string())?;
    let tasks_array = raw_tasks
        .as_array()
        .ok_or_else(|| "tasks must be an array".to_string())?;

    if tasks_array.is_empty() {
        return Err("tasks must contain at least one task".to_string());
    }

    tasks_array
        .iter()
        .enumerate()
        .map(|(index, task)| parse_task(index, task))
        .collect()
}

fn parse_task(index: usize, value: &Value) -> Result<ValidatedTaskEstimate, String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("tasks[{index}] must be a JSON object"))?;

    let title = parse_non_empty_string_field(object.get("title"), "title", Some(index))?;
    let description =
        parse_non_empty_string_field(object.get("description"), "description", Some(index))?;
    let price_sol = parse_price_field(object.get("price_sol"), Some(index))?;
    let complexity = parse_complexity_field(object.get("complexity"), Some(index))?;
    let rationale =
        parse_non_empty_string_field(object.get("rationale"), "rationale", Some(index))?;

    Ok(ValidatedTaskEstimate {
        title,
        description,
        price_sol,
        complexity,
        rationale,
    })
}

fn parse_non_empty_string_field(
    value: Option<&Value>,
    field: &str,
    task_index: Option<usize>,
) -> Result<String, String> {
    let prefix = task_index
        .map(|index| format!("tasks[{index}].{field}"))
        .unwrap_or_else(|| field.to_string());

    let raw = value.ok_or_else(|| format!("missing required field: {prefix}"))?;
    let parsed = raw
        .as_str()
        .ok_or_else(|| format!("{prefix} must be a string"))?
        .trim()
        .to_string();

    if parsed.is_empty() {
        return Err(format!("{prefix} must not be blank"));
    }

    Ok(parsed)
}

fn parse_price_field(
    price_value: Option<&Value>,
    task_index: Option<usize>,
) -> Result<f64, String> {
    let field_name = task_index
        .map(|index| format!("tasks[{index}].price_sol"))
        .unwrap_or_else(|| "price_sol".to_string());

    let raw_price = price_value.ok_or_else(|| format!("missing required field: {field_name}"))?;

    let price = match raw_price {
        Value::Number(number) => number
            .as_f64()
            .ok_or_else(|| format!("{field_name} must be numeric"))?,
        Value::String(raw_number) => raw_number
            .trim()
            .parse::<f64>()
            .map_err(|_| format!("{field_name} string value must be numeric"))?,
        _ => return Err(format!("{field_name} must be numeric")),
    };

    if !price.is_finite() || !(MIN_PRICE_SOL..=MAX_PRICE_SOL).contains(&price) {
        return Err(format!(
            "{field_name} must be within [{MIN_PRICE_SOL}, {MAX_PRICE_SOL}]"
        ));
    }

    Ok(round_two_decimals(price))
}

fn parse_complexity_field(
    complexity_value: Option<&Value>,
    task_index: Option<usize>,
) -> Result<u8, String> {
    let field_name = task_index
        .map(|index| format!("tasks[{index}].complexity"))
        .unwrap_or_else(|| "complexity".to_string());

    let raw_complexity =
        complexity_value.ok_or_else(|| format!("missing required field: {field_name}"))?;

    let complexity = match raw_complexity {
        Value::Number(number) => {
            if let Some(as_u64) = number.as_u64() {
                i64::try_from(as_u64).map_err(|_| format!("{field_name} must be an integer"))?
            } else if let Some(as_i64) = number.as_i64() {
                as_i64
            } else {
                return Err(format!("{field_name} must be an integer"));
            }
        }
        Value::String(raw_number) => raw_number
            .trim()
            .parse::<i64>()
            .map_err(|_| format!("{field_name} string value must be an integer"))?,
        _ => return Err(format!("{field_name} must be an integer")),
    };

    if !(MIN_COMPLEXITY..=MAX_COMPLEXITY).contains(&complexity) {
        return Err(format!(
            "{field_name} must be in range [{MIN_COMPLEXITY}, {MAX_COMPLEXITY}]"
        ));
    }

    u8::try_from(complexity)
        .map_err(|_| format!("{field_name} must be in range [{MIN_COMPLEXITY}, {MAX_COMPLEXITY}]"))
}

fn parse_optional_rationale_field(
    rationale_value: Option<&Value>,
    task_count: usize,
) -> Result<String, String> {
    match rationale_value {
        Some(value) => parse_non_empty_string_field(Some(value), "rationale", None),
        None => Ok(format!("Projekt podzielono na {task_count} task(i).")),
    }
}

fn build_fallback_estimate(task_description: &str, failure_reason: &str) -> ValidatedEstimate {
    let word_count = task_description
        .split_whitespace()
        .filter(|segment| !segment.trim().is_empty())
        .count();
    let normalized_reason = normalize_failure_reason(failure_reason);

    let overall_complexity = fallback_complexity(word_count);
    let bounded_words = word_count.min(FALLBACK_MAX_WORDS) as f64;
    let estimated_total = (f64::from(overall_complexity) * FALLBACK_BASE_PRICE_PER_COMPLEXITY)
        + (bounded_words * FALLBACK_PRICE_PER_WORD);
    let total_price_sol = round_two_decimals(estimated_total.clamp(MIN_PRICE_SOL, MAX_PRICE_SOL));

    let analysis_price = round_two_decimals(total_price_sol * 0.2);
    let implementation_price = round_two_decimals(total_price_sol * 0.6);
    let qa_price =
        round_two_decimals((total_price_sol - analysis_price - implementation_price).max(1.0));

    let analysis_complexity = overall_complexity.saturating_sub(1).max(1);
    let qa_complexity = (overall_complexity + 1).min(5);

    let tasks = vec![
        ValidatedTaskEstimate {
            title: "Analiza wymagań".to_string(),
            description: "Doprecyzowanie zakresu i podział projektu na etapy realizacji."
                .to_string(),
            price_sol: analysis_price,
            complexity: analysis_complexity,
            rationale: "Task obejmuje analizę i przygotowanie planu realizacji.".to_string(),
        },
        ValidatedTaskEstimate {
            title: "Implementacja".to_string(),
            description: "Wykonanie głównej funkcjonalności projektu zgodnie z opisem.".to_string(),
            price_sol: implementation_price,
            complexity: overall_complexity,
            rationale: "To rdzeń projektu wymagający najwięcej pracy inżynierskiej.".to_string(),
        },
        ValidatedTaskEstimate {
            title: "Testy i odbiór".to_string(),
            description: "Walidacja jakości, poprawki i przygotowanie do przekazania.".to_string(),
            price_sol: qa_price,
            complexity: qa_complexity,
            rationale: "Task domyka realizację i minimalizuje ryzyko regresji.".to_string(),
        },
    ];

    ValidatedEstimate {
        tasks,
        total_price_sol,
        overall_complexity,
        rationale: format!(
            "Użyto fallbacku estymacji, ponieważ odpowiedź modelu była niepoprawna ({normalized_reason}). \
             Projekt podzielono deterministycznie na taski na podstawie długości opisu ({word_count} słów)."
        ),
    }
}

fn normalize_failure_reason(reason: &str) -> &str {
    let trimmed_reason = reason.trim();
    if trimmed_reason.is_empty() {
        "nieznany błąd parsera"
    } else {
        trimmed_reason
    }
}

fn fallback_complexity(word_count: usize) -> u8 {
    match word_count {
        0..=6 => 1,
        7..=12 => 2,
        13..=24 => 3,
        25..=40 => 4,
        _ => 5,
    }
}

fn round_two_decimals(value: f64) -> f64 {
    (value * PRICE_PRECISION_SCALE).round() / PRICE_PRECISION_SCALE
}

#[cfg(test)]
mod tests {
    use super::parse_and_validate_estimate_output;

    #[test]
    fn valid_json_is_passed_through_without_fallback() {
        let output = r#"{
          "tasks": [
            {
              "title": "Analiza",
              "description": "Rozbicie zakresu.",
              "price_sol": 120.5,
              "complexity": 2,
              "rationale": "Niski poziom ryzyka."
            },
            {
              "title": "Implementacja",
              "description": "Kod i integracje.",
              "price_sol": 300,
              "complexity": 4,
              "rationale": "Największy nakład pracy."
            }
          ],
          "rationale": "Podział na 2 etapy."
        }"#;

        let result = parse_and_validate_estimate_output("Duży projekt integracyjny.", output);

        assert!(!result.used_fallback);
        assert_eq!(result.estimate.tasks.len(), 2);
        assert_eq!(result.estimate.total_price_sol, 420.5);
        assert_eq!(result.estimate.overall_complexity, 4);
        assert_eq!(result.estimate.tasks[0].title, "Analiza");
    }

    #[test]
    fn malformed_json_uses_deterministic_fallback() {
        let output = "nie zwracam jsona";

        let result = parse_and_validate_estimate_output("Dodaj endpoint i testy.", output);

        assert!(result.used_fallback);
        assert_eq!(result.estimate.tasks.len(), 3);
        assert!(
            result
                .estimate
                .rationale
                .contains("Użyto fallbacku estymacji")
        );
        assert!((1.0..=100_000.0).contains(&result.estimate.total_price_sol));
    }

    #[test]
    fn invalid_task_complexity_uses_fallback_with_explicit_reason() {
        let output = r#"{
          "tasks": [{
            "title": "Implementacja",
            "description": "Kod",
            "price_sol": 500,
            "complexity": 8,
            "rationale": "Za wysoko."
          }]
        }"#;

        let result = parse_and_validate_estimate_output("Integracja API.", output);

        assert!(result.used_fallback);
        assert!(
            result
                .estimate
                .rationale
                .contains("tasks[0].complexity must be in range [1, 5]")
        );
    }

    #[test]
    fn parses_json_embedded_in_text() {
        let output = "Wynik:\n{\"tasks\":[{\"title\":\"A\",\"description\":\"B\",\"price_sol\":\"90\",\"complexity\":\"2\",\"rationale\":\"ok\"}]}\nDziękuję.";

        let result = parse_and_validate_estimate_output("Małe zadanie.", output);

        assert!(!result.used_fallback);
        assert_eq!(result.estimate.tasks.len(), 1);
        assert_eq!(result.estimate.tasks[0].price_sol, 90.0);
    }

    #[test]
    fn missing_tasks_field_uses_fallback_with_explicit_reason() {
        let output = r#"{"rationale":"brak tasks"}"#;

        let result = parse_and_validate_estimate_output("Dodaj endpoint.", output);

        assert!(result.used_fallback);
        assert!(
            result
                .estimate
                .rationale
                .contains("missing required field: tasks")
        );
    }

    #[test]
    fn fallback_is_deterministic_for_same_input() {
        let task = "Dodaj endpoint i testy integracyjne dla nowego API.";
        let output = "brak jsona";

        let first = parse_and_validate_estimate_output(task, output);
        let second = parse_and_validate_estimate_output(task, output);

        assert!(first.used_fallback);
        assert!(second.used_fallback);
        assert_eq!(first.estimate, second.estimate);
    }
}
