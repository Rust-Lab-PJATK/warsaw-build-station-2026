const SYSTEM_PROMPT: &str = r#"Jesteś ekspertem od wyceny zadań software house'u.
Twoim celem jest oszacowanie pojedynczego zadania na podstawie opisu.
Zwracaj cenę wyłącznie w SOL oraz poziom złożoności w skali 1-5.
Odpowiadaj wyłącznie poprawnym JSON-em o strukturze:
{"price_sol": <liczba dodatnia>, "complexity": <1-5>, "rationale": "<krótkie uzasadnienie>"}.
Uzasadnienie ma mieć maksymalnie 2 krótkie zdania i być po polsku."#;

#[derive(Debug, Clone, Copy)]
struct FewShotExample {
    description: &'static str,
    price_sol: &'static str,
    complexity: u8,
    rationale: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct HistoricalTransaction {
    reference: &'static str,
    summary: &'static str,
    price_sol: &'static str,
    complexity: u8,
}

const FEW_SHOT_EXAMPLES: [FewShotExample; 6] = [
    FewShotExample {
        description: "Dodanie formularza kontaktowego z walidacją pól i wysyłką maila.",
        price_sol: "180.0",
        complexity: 2,
        rationale: "Zakres jest mały i opiera się o standardowe komponenty backend/frontend.",
    },
    FewShotExample {
        description: "Integracja płatności Stripe z webhookami i obsługą nieudanych płatności.",
        price_sol: "690.0",
        complexity: 4,
        rationale: "Wymaga integracji zewnętrznego API i bezpiecznej obsługi asynchronicznych zdarzeń.",
    },
    FewShotExample {
        description: "Refaktoryzacja modułu logowania i dodanie resetu hasła przez e-mail.",
        price_sol: "420.0",
        complexity: 3,
        rationale: "Zmiana obejmuje logikę autoryzacji i nowy przepływ użytkownika, ale bez migracji danych.",
    },
    FewShotExample {
        description: "Wdrożenie wyszukiwania pełnotekstowego ofert z filtrowaniem i paginacją.",
        price_sol: "810.0",
        complexity: 4,
        rationale: "Potrzebna jest optymalizacja zapytań oraz spójna implementacja API i UI filtrów.",
    },
    FewShotExample {
        description: "Naprawa błędu eksportu CSV powodującego zły separator i brak polskich znaków.",
        price_sol: "130.0",
        complexity: 1,
        rationale: "To punktowa poprawka z ograniczonym wpływem na resztę systemu.",
    },
    FewShotExample {
        description: "Zbudowanie panelu administracyjnego do zarządzania cennikiem i uprawnieniami ról.",
        price_sol: "1_250.0",
        complexity: 5,
        rationale: "Zakres obejmuje wiele ekranów, role użytkowników i ryzyko regresji w krytycznych obszarach.",
    },
];

const HISTORICAL_TRANSACTIONS: [HistoricalTransaction; 6] = [
    HistoricalTransaction {
        reference: "TX-2026-001",
        summary: "Dodanie endpointu REST do pobierania statusu zamówienia.",
        price_sol: "210.0",
        complexity: 2,
    },
    HistoricalTransaction {
        reference: "TX-2026-007",
        summary: "Integracja z Slack webhook do powiadomień o incydentach.",
        price_sol: "300.0",
        complexity: 2,
    },
    HistoricalTransaction {
        reference: "TX-2026-014",
        summary: "Migracja bazy danych klientów z mapowaniem pól i walidacją rekordów.",
        price_sol: "980.0",
        complexity: 5,
    },
    HistoricalTransaction {
        reference: "TX-2026-019",
        summary: "Dodanie cache Redis dla listy ofert i strategii wygaszania.",
        price_sol: "560.0",
        complexity: 3,
    },
    HistoricalTransaction {
        reference: "TX-2026-021",
        summary: "Implementacja uploadu plików do S3 z podpisanymi URL-ami.",
        price_sol: "640.0",
        complexity: 4,
    },
    HistoricalTransaction {
        reference: "TX-2026-027",
        summary: "Rozbudowa raportu KPI z dodatkowymi metrykami i eksportem PDF.",
        price_sol: "760.0",
        complexity: 4,
    },
];

pub fn build_estimation_prompt(task_description: &str) -> String {
    let normalized_description = task_description.trim();

    [
        "### INSTRUKCJA SYSTEMOWA".to_string(),
        SYSTEM_PROMPT.to_string(),
        String::new(),
        "### KONTEKST HISTORYCZNY (MVP)".to_string(),
        format_historical_transactions(),
        String::new(),
        "### PRZYKŁADY FEW-SHOT".to_string(),
        format_few_shot_examples(),
        String::new(),
        "### NOWE ZADANIE DO WYCENY".to_string(),
        format!("Opis: {normalized_description}"),
        "Zwróć wyłącznie JSON zgodny z instrukcją systemową.".to_string(),
    ]
    .join("\n")
}

fn format_historical_transactions() -> String {
    HISTORICAL_TRANSACTIONS
        .iter()
        .map(|transaction| {
            format!(
                "- {} | opis: {} | cena: {} SOL | complexity: {}",
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
                "Przykład {}:\n\
                 Wejście: {}\n\
                 Wyjście: {{\"price_sol\": {}, \"complexity\": {}, \"rationale\": \"{}\"}}",
                index + 1,
                example.description,
                example.price_sol,
                example.complexity,
                example.rationale
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
        let prompt = build_estimation_prompt("Dodanie endpointu do aktualizacji profilu.");

        assert!(prompt.contains("### INSTRUKCJA SYSTEMOWA"));
        assert!(prompt.contains("### KONTEKST HISTORYCZNY (MVP)"));
        assert!(prompt.contains("### PRZYKŁADY FEW-SHOT"));
        assert!(prompt.contains("### NOWE ZADANIE DO WYCENY"));
    }

    #[test]
    fn estimation_prompt_embeds_task_description() {
        let prompt = build_estimation_prompt("Nowa integracja fakturowania VAT UE.");

        assert!(prompt.contains("Opis: Nowa integracja fakturowania VAT UE."));
    }
}
