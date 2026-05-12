import { NextRequest, NextResponse } from "next/server";

const BACKEND = process.env.BACKEND_URL ?? "http://localhost:5150";

export async function POST(req: NextRequest) {
  const body = await req.json().catch(() => ({}));
  const description: string = body.description ?? "";

  if (!description.trim()) {
    return NextResponse.json({ error: "description required" }, { status: 400 });
  }

  let upstream: Response;
  try {
    upstream = await fetch(`${BACKEND}/api/estimate`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ task_description: description }),
    });
  } catch {
    return NextResponse.json({ error: "Backend unreachable" }, { status: 502 });
  }

  if (!upstream.ok) {
    return NextResponse.json({ error: "Estimate failed" }, { status: 502 });
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const data: any = await upstream.json();

  const c: number = data.overall_complexity ?? 3;
  const complexity = c <= 2 ? "low" : c <= 4 ? "medium" : "high";
  const advance_bps = c <= 2 ? 1000 : c <= 4 ? 2000 : 3000;
  const estimated_duration = c <= 2 ? "2–4 hours" : c <= 4 ? "1–2 days" : "3–5 days";

  return NextResponse.json({
    price_sol: data.total_price_sol ?? 0,
    advance_bps,
    complexity,
    reasoning: data.rationale ?? "",
    estimated_duration,
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    similar: (data.tasks ?? []).slice(0, 3).map((t: any) => ({
      description: t.title,
      price_sol: t.price_sol,
      complexity: t.complexity <= 2 ? "low" : t.complexity <= 4 ? "medium" : "high",
    })),
  });
}
