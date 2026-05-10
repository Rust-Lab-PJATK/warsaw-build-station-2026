use serde_json::Value;

const MIN_PRICE_USDC: f64 = 1.0;
const MAX_PRICE_USDC: f64 = 100_000.0;
const PRICE_PRECISION_SCALE: f64 = 100.0;
const MIN_COMPLEXITY: i64 = 1;
const MAX_COMPLEXITY: i64 = 5;
const FALLBACK_BASE_PRICE_PER_COMPLEXITY: f64 = 220.0;
const FALLBACK_PRICE_PER_WORD: f64 = 6.0;
const FALLBACK_MAX_WORDS: usize = 80;

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedEstimate {
    pub price_usdc: f64,
    pub complexity: u8,
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
                if depth == 0 {
                    if let Some(start) = start_index.take() {
                        if let Some(slice) = text.get(start..=index) {
                            slices.push(slice);
                        }
                    }
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

    let price = parse_price_field(object.get("price_usdc"))?;
    let complexity = parse_complexity_field(object.get("complexity"))?;
    let rationale = parse_rationale_field(object.get("rationale"))?;

    Ok(ValidatedEstimate {
        price_usdc: price,
        complexity,
        rationale,
    })
}

fn parse_price_field(price_value: Option<&Value>) -> Result<f64, String> {
    let raw_price = price_value.ok_or_else(|| "missing required field: price_usdc".to_string())?;

    let price = match raw_price {
        Value::Number(number) => number
            .as_f64()
            .ok_or_else(|| "price_usdc must be numeric".to_string())?,
        Value::String(raw_number) => raw_number
            .trim()
            .parse::<f64>()
            .map_err(|_| "price_usdc string value must be numeric".to_string())?,
        _ => return Err("price_usdc must be numeric".to_string()),
    };

    if !price.is_finite() || !(MIN_PRICE_USDC..=MAX_PRICE_USDC).contains(&price) {
        return Err(format!(
            "price_usdc must be within [{MIN_PRICE_USDC}, {MAX_PRICE_USDC}]"
        ));
    }

    Ok(round_two_decimals(price))
}

fn parse_complexity_field(complexity_value: Option<&Value>) -> Result<u8, String> {
    let raw_complexity =
        complexity_value.ok_or_else(|| "missing required field: complexity".to_string())?;

    let complexity = match raw_complexity {
        Value::Number(number) => {
            if let Some(as_u64) = number.as_u64() {
                i64::try_from(as_u64).map_err(|_| "complexity must be an integer".to_string())?
            } else if let Some(as_i64) = number.as_i64() {
                as_i64
            } else {
                return Err("complexity must be an integer".to_string());
            }
        }
        Value::String(raw_number) => raw_number
            .trim()
            .parse::<i64>()
            .map_err(|_| "complexity string value must be an integer".to_string())?,
        _ => return Err("complexity must be an integer".to_string()),
    };

    if !(MIN_COMPLEXITY..=MAX_COMPLEXITY).contains(&complexity) {
        return Err(format!(
            "complexity must be in range [{MIN_COMPLEXITY}, {MAX_COMPLEXITY}]"
        ));
    }

    u8::try_from(complexity)
        .map_err(|_| format!("complexity must be in range [{MIN_COMPLEXITY}, {MAX_COMPLEXITY}]"))
}

fn parse_rationale_field(rationale_value: Option<&Value>) -> Result<String, String> {
    let raw_rationale =
        rationale_value.ok_or_else(|| "missing required field: rationale".to_string())?;
    let rationale = raw_rationale
        .as_str()
        .ok_or_else(|| "rationale must be a string".to_string())?
        .trim()
        .to_string();

    if rationale.is_empty() {
        return Err("rationale must not be blank".to_string());
    }

    Ok(rationale)
}

fn build_fallback_estimate(task_description: &str, failure_reason: &str) -> ValidatedEstimate {
    let word_count = task_description
        .split_whitespace()
        .filter(|segment| !segment.trim().is_empty())
        .count();
    let normalized_reason = normalize_failure_reason(failure_reason);

    let complexity = fallback_complexity(word_count);
    let bounded_words = word_count.min(FALLBACK_MAX_WORDS) as f64;
    let estimated_price = (f64::from(complexity) * FALLBACK_BASE_PRICE_PER_COMPLEXITY)
        + (bounded_words * FALLBACK_PRICE_PER_WORD);
    let bounded_price = estimated_price.clamp(MIN_PRICE_USDC, MAX_PRICE_USDC);
    let price_usdc = round_two_decimals(bounded_price);

    ValidatedEstimate {
        price_usdc,
        complexity,
        rationale: format!(
            "Użyto fallbacku estymacji, ponieważ odpowiedź modelu była niepoprawna ({normalized_reason}). \
              Szacunek oparto deterministycznie na długości opisu zadania ({word_count} słów)."
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
        let output =
            r#"{"price_usdc": 320.5, "complexity": 3, "rationale": "Zakres jest średni."}"#;

        let result = parse_and_validate_estimate_output("Dodaj endpoint i testy.", output);

        assert!(!result.used_fallback);
        assert_eq!(result.estimate.price_usdc, 320.5);
        assert_eq!(result.estimate.complexity, 3);
        assert_eq!(result.estimate.rationale, "Zakres jest średni.");
    }

    #[test]
    fn malformed_json_uses_deterministic_fallback() {
        let output = "nie zwracam jsona";

        let result = parse_and_validate_estimate_output("Dodaj endpoint i testy.", output);

        assert!(result.used_fallback);
        assert!(
            result
                .estimate
                .rationale
                .contains("Użyto fallbacku estymacji")
        );
        assert!(result.estimate.rationale.contains("niepoprawna"));
        assert!((1.0..=100_000.0).contains(&result.estimate.price_usdc));
        assert!((1..=5).contains(&i32::from(result.estimate.complexity)));
    }

    #[test]
    fn out_of_range_complexity_uses_fallback_with_explicit_reason() {
        let output =
            r#"{"price_usdc": 600, "complexity": 8, "rationale": "Model podał zły poziom."}"#;

        let result = parse_and_validate_estimate_output(
            "Integracja z płatnościami, logowanie i panel administracyjny.",
            output,
        );

        assert!(result.used_fallback);
        assert!(
            result
                .estimate
                .rationale
                .contains("complexity must be in range [1, 5]")
        );
        assert!((1..=5).contains(&i32::from(result.estimate.complexity)));
    }

    #[test]
    fn parses_json_embedded_in_text() {
        let output = "Oto wynik:\n{\"price_usdc\": \"400.0\", \"complexity\": \"4\", \"rationale\": \"Wymaga integracji.\"}\nDziękuję.";

        let result = parse_and_validate_estimate_output("Integracja API.", output);

        assert!(!result.used_fallback);
        assert_eq!(result.estimate.price_usdc, 400.0);
        assert_eq!(result.estimate.complexity, 4);
    }

    #[test]
    fn parses_first_valid_json_object_when_text_contains_invalid_braces_before_it() {
        let output = "Szkic: {not-json}\nFinalna odpowiedź:\n{\"price_usdc\": 410, \"complexity\": 3, \"rationale\": \"To jest poprawna odpowiedź.\"}";

        let result = parse_and_validate_estimate_output("Integracja API.", output);

        assert!(!result.used_fallback);
        assert_eq!(result.estimate.price_usdc, 410.0);
        assert_eq!(result.estimate.complexity, 3);
        assert_eq!(result.estimate.rationale, "To jest poprawna odpowiedź.");
    }

    #[test]
    fn price_is_normalized_to_two_decimal_places() {
        let output =
            r#"{"price_usdc": 123.456, "complexity": 2, "rationale": "Zakres jest mały."}"#;

        let result = parse_and_validate_estimate_output("Dodaj endpoint.", output);

        assert!(!result.used_fallback);
        assert_eq!(result.estimate.price_usdc, 123.46);
    }

    #[test]
    fn fractional_complexity_uses_fallback() {
        let output =
            r#"{"price_usdc": 123.45, "complexity": 2.5, "rationale": "Zakres jest mały."}"#;

        let result = parse_and_validate_estimate_output("Dodaj endpoint.", output);

        assert!(result.used_fallback);
        assert!(
            result
                .estimate
                .rationale
                .contains("complexity must be an integer")
        );
    }

    #[test]
    fn blank_rationale_uses_fallback() {
        let output = r#"{"price_usdc": 123.45, "complexity": 2, "rationale": "   "}"#;

        let result = parse_and_validate_estimate_output("Dodaj endpoint.", output);

        assert!(result.used_fallback);
        assert!(
            result
                .estimate
                .rationale
                .contains("rationale must not be blank")
        );
    }

    #[test]
    fn missing_required_field_uses_fallback_with_explicit_reason() {
        let output = r#"{"price_usdc": 123.45, "rationale": "Brakuje complexity"}"#;

        let result = parse_and_validate_estimate_output("Dodaj endpoint.", output);

        assert!(result.used_fallback);
        assert!(
            result
                .estimate
                .rationale
                .contains("missing required field: complexity")
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
