// Backend API client — kontrakt.md is the source of truth
// Base URL: http://localhost:5150 (dev)

const BACKEND_URL =
  process.env.NEXT_PUBLIC_BACKEND_URL ?? "http://localhost:5150";

// ─── Types ──────────────────────────────────────────────────────────────────

export interface BackendJobTask {
  title: string;
  description: string;
  price_sol: number;
  complexity: number; // 1–5
  rationale: string;
}

export interface BackendEstimateResponse {
  job_id: string | null;
  tasks: BackendJobTask[];
  total_price_sol: number;
  overall_complexity: number; // 1–5
  rationale: string;
}

export type JobStatus =
  | "PRICED"
  | "DEPOSIT"
  | "ADVANCE"
  | "AWAITING_REVIEW"
  | "ACCEPTED";

export interface BackendJob {
  id: string;
  status: JobStatus;
  task_pubkey: string | null;
  tasks: BackendJobTask[];
  total_price_sol: number;
  overall_complexity: number;
  rationale: string;
  created_at: string;
  updated_at: string;
}

export interface BackendValidationError {
  code: "validation_error";
  message: string;
  field_errors: Record<string, string[]>;
}

export interface BackendServiceError {
  code: string;
  message: string;
}

// ─── Helpers ────────────────────────────────────────────────────────────────

export async function estimateTask(
  taskDescription: string
): Promise<BackendEstimateResponse> {
  const res = await fetch(`${BACKEND_URL}/api/estimate`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ task_description: taskDescription }),
  });

  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(
      (err as BackendServiceError).message ?? `HTTP ${res.status}`
    );
  }

  return res.json() as Promise<BackendEstimateResponse>;
}

export async function linkTask(
  jobId: string,
  taskPubkey: string
): Promise<{ id: string; task_pubkey: string }> {
  const res = await fetch(`${BACKEND_URL}/api/jobs/${jobId}/link-task`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ task_pubkey: taskPubkey }),
  });

  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(
      (err as BackendServiceError).message ?? `HTTP ${res.status}`
    );
  }

  return res.json();
}

export async function submitPreview(jobId: string): Promise<BackendJob> {
  const res = await fetch(`${BACKEND_URL}/api/task/${jobId}/preview`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
  });

  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(
      (err as BackendServiceError).message ?? `HTTP ${res.status}`
    );
  }

  return res.json() as Promise<BackendJob>;
}

// ─── localStorage helpers ────────────────────────────────────────────────────
// Maps Solana task pubkey → MongoDB job_id for demo use

const STORAGE_KEY = "task_job_map";

export function storeJobId(taskPubkey: string, jobId: string): void {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    const map: Record<string, string> = raw ? JSON.parse(raw) : {};
    map[taskPubkey] = jobId;
    localStorage.setItem(STORAGE_KEY, JSON.stringify(map));
  } catch {
    // silently ignore
  }
}

export function getJobId(taskPubkey: string): string | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const map: Record<string, string> = JSON.parse(raw);
    return map[taskPubkey] ?? null;
  } catch {
    return null;
  }
}
