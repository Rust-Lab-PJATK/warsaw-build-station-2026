import { estimateTask } from "@/lib/backendApi";

export async function POST(req: Request) {
  try {
    const body = await req.json().catch(() => null);
    const description: string = body?.description ?? "";

    if (!description || description.trim().length < 10) {
      return Response.json({ error: "Description too short" }, { status: 400 });
    }

    const data = await estimateTask(description.trim());
    return Response.json(data);
  } catch (e) {
    console.error("estimate-price error:", e);
    return Response.json({ error: "Estimation failed" }, { status: 500 });
  }
}
