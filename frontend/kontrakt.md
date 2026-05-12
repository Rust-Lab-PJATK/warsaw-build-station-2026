# API Contract — Backend ↔ Frontend

> Dokument opisuje wszystkie endpointy backendu. Traktuj jako źródło prawdy przy integracji frontendu.
> Base URL: `http://localhost:5150` (dev)

---

## Spis endpointów

| #   | Method | Path                      | Cel                               |
| --- | ------ | ------------------------- | --------------------------------- |
| 1   | `GET`  | `/api`                    | Health check                      |
| 2   | `POST` | `/api/estimate`           | Wygeneruj wycenę zadania przez AI |
| 3   | `POST` | `/api/jobs/:id/link-task` | Przypisz Solana pubkey do joba    |
| 4   | `POST` | `/api/task/:id/preview`   | Oznacz job jako "Proof submitted" |

---

## 1. `GET /api` — Health check

Sprawdza czy serwer żyje.

**Request:** brak body, brak parametrów.

**Response `200 OK`:**

```json
{
  "app_name": "loco"
}
```

---

## 2. `POST /api/estimate` — Wycena zadania przez AI

Wysyłasz opis zadania, dostajesz podział na podzadania z cenami, złożonością i uzasadnieniem. Endpoint automatycznie tworzy rekord `Job` w MongoDB (status: `PRICED`).

**Request body (`application/json`):**

```typescript
{
  task_description: string; // wymagany, nie może być pusty
}
```

**Przykład:**

```json
{
  "task_description": "Audit Solana program for reentrancy and integer overflow vulnerabilities"
}
```

**Response `200 OK`:**

```typescript
{
  tasks: Array<{
    title: string,        // nazwa podzadania
    description: string,  // opis
    price_sol: number,    // cena w SOL, 2 miejsca dziesiętne, zakres [1.0, 100000.0]
    complexity: number,   // 1–5 (1=łatwe, 5=bardzo trudne)
    rationale: string     // uzasadnienie ceny/złożoności
  }>,
  total_price_sol: number,      // suma cen wszystkich podzadań, 2 dec.
  overall_complexity: number,   // maks. złożoność ze wszystkich tasków (1–5)
  rationale: string             // ogólne uzasadnienie podziału
}
```

**Przykład response:**

```json
{
  "tasks": [
    {
      "title": "Reentrancy Analysis",
      "description": "Check all cross-program invocations for reentrancy patterns",
      "price_sol": 150.0,
      "complexity": 4,
      "rationale": "Requires deep CPI trace analysis"
    },
    {
      "title": "Integer Overflow Detection",
      "description": "Scan arithmetic operations for unchecked overflows",
      "price_sol": 120.0,
      "complexity": 3,
      "rationale": "Systematic but time-intensive review"
    },
    {
      "title": "Final Report",
      "description": "Written report with severity ratings",
      "price_sol": 80.0,
      "complexity": 2,
      "rationale": "Documentation effort"
    }
  ],
  "total_price_sol": 350.0,
  "overall_complexity": 4,
  "rationale": "Security audit of moderate-to-high complexity Solana program"
}
```

**Błędy:**

| Status | `code`                                  | Kiedy                                               |
| ------ | --------------------------------------- | --------------------------------------------------- |
| `400`  | `validation_error`                      | Brak / puste `task_description`, nieprawidłowy JSON |
| `502`  | `estimate_provider_request_failed`      | LLM API nie odpowiedziało                           |
| `503`  | `estimate_provider_configuration_error` | Brak klucza `OPENAI_API_KEY` / `ELEVENLABS_API_KEY` |

**Validation error `400`:**

```typescript
{
  code: "validation_error",
  message: "Validation failed",   // lub "request body must be valid JSON" / "request body must be a JSON object"
  field_errors: {
    task_description: ["must not be blank"]  // tablica komunikatów
  }
}
```

**Service error `502` / `503`:**

```typescript
{
  code: string,    // "estimate_provider_request_failed" | "estimate_provider_configuration_error"
  message: string  // opis błędu
}
```

> **Uwaga:** Jeśli LLM zwróci nieprawidłową odpowiedź (np. brakuje pól), backend generuje deterministyczny fallback — zawsze 3 zadania: `Analysis → Implementation → QA/Testing`. Frontend powinien to wyświetlić normalnie.

---

## 3. `POST /api/jobs/:id/link-task` — Przypisz Solana pubkey

Po stworzeniu joba na blockchainie (klient podpisał tx) przypisz on-chain pubkey do rekordu w MongoDB. Nie zmienia statusu joba.

**Path params:**

```
id: string  // MongoDB ObjectId — 24-znakowy hex, np. "6642f3a1b2c3d4e5f6a7b8c9"
```

**Request body (`application/json`):**

```typescript
{
  task_pubkey: string; // wymagany, prawidłowy Solana pubkey (base58)
}
```

**Przykład:**

```json
{
  "task_pubkey": "8xKpQm3mNrAbCd4eFgHiJk5lMnOpQrSt6uVwXyZaBcD"
}
```

**Response `200 OK`:**

```typescript
{
  id: string,           // MongoDB ObjectId joba
  task_pubkey: string   // przypisany Solana pubkey
}
```

**Przykład response:**

```json
{
  "id": "6642f3a1b2c3d4e5f6a7b8c9",
  "task_pubkey": "8xKpQm3mNrAbCd4eFgHiJk5lMnOpQrSt6uVwXyZaBcD"
}
```

**Błędy:**

| Status | `code`             | Kiedy                                                        |
| ------ | ------------------ | ------------------------------------------------------------ |
| `400`  | `validation_error` | Brak `task_pubkey`, puste, nieprawidłowy pubkey lub zły JSON |
| `404`  | `job_not_found`    | Job o podanym `id` nie istnieje                              |

**Validation error `400`:**

```typescript
{
  code: "validation_error",
  message: "Validation failed",
  field_errors: {
    task_pubkey: ["is required"]                   // pole pominięte
                | ["must not be blank"]             // puste string
                | ["must be a valid Solana pubkey"] // błędny base58
  }
}
```

**Not found `404`:**

```typescript
{
  code: "job_not_found",
  message: "job not found"
}
```

---

## 4. `POST /api/task/:id/preview` — Zgłoś Proof of Work

Agent zgłasza, że skończył robotę. Zmienia status joba na `AWAITING_REVIEW`.

**Path params:**

```
id: string  // MongoDB ObjectId — 24-znakowy hex
```

**Request body:** brak (puste body).

**Response `200 OK`:**

```typescript
{
  id: string,
  status: "AWAITING_REVIEW",
  task_pubkey: string | null,
  tasks: Array<{
    title: string,
    description: string,
    price_sol: number,
    complexity: number,
    rationale: string
  }>,
  total_price_sol: number,
  overall_complexity: number,
  rationale: string,
  created_at: string,   // ISO 8601, np. "2026-05-11T14:23:00Z"
  updated_at: string    // ISO 8601 — zaktualizowane do momentu wywołania
}
```

**Błędy:**

| Status | `code`          | Kiedy                           |
| ------ | --------------- | ------------------------------- |
| `404`  | `job_not_found` | Job o podanym `id` nie istnieje |

**Not found `404`:**

```typescript
{
  code: "job_not_found",
  message: "job not found"
}
```

---

## Typy pomocnicze

### `JobStatus` — możliwe wartości pola `status`

| Wartość           | Znaczenie                                      |
| ----------------- | ---------------------------------------------- |
| `PRICED`          | Wycena gotowa (tworzony przez `/api/estimate`) |
| `DEPOSIT`         | Klient wpłacił depozyt                         |
| `ADVANCE`         | Zaliczka wysłana do agenta                     |
| `AWAITING_REVIEW` | Agent zgłosił proof of work                    |
| `ACCEPTED`        | Klient zaakceptował wyniki                     |

### `Job` — pełny obiekt (zwracany przez endpoint 4)

```typescript
interface Job {
  id: string;
  status: JobStatus;
  task_pubkey: string | null;
  tasks: JobTask[];
  total_price_sol: number;
  overall_complexity: number; // 1–5
  rationale: string;
  created_at: string; // ISO 8601
  updated_at: string; // ISO 8601
}

interface JobTask {
  title: string;
  description: string;
  price_sol: number; // [1.0, 100000.0], 2 dec.
  complexity: number; // 1–5
  rationale: string;
}
```

---
