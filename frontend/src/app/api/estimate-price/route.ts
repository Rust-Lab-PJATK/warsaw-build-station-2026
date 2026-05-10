import Anthropic from "@anthropic-ai/sdk";

const client = new Anthropic();

const HISTORICAL_TASKS = [
  {
    description: "Summarize a 10-page research PDF and extract key points",
    price_sol: 0.05,
    complexity: "low",
    duration: "1-2h",
  },
  {
    description: "Build a Python script to scrape and parse website data into CSV",
    price_sol: 0.35,
    complexity: "medium",
    duration: "3-6h",
  },
  {
    description: "Create a REST API with JWT authentication and PostgreSQL integration",
    price_sol: 1.2,
    complexity: "medium",
    duration: "1-2d",
  },
  {
    description: "Train a custom image classification model on provided dataset",
    price_sol: 2.5,
    complexity: "high",
    duration: "2-4d",
  },
  {
    description: "Conduct a full security audit of a Solana smart contract with report",
    price_sol: 4.8,
    complexity: "high",
    duration: "3-5d",
  },
];

export async function POST(req: Request) {
  try {
    const { description } = await req.json();

    if (!description || description.trim().length < 10) {
      return Response.json({ error: "Description too short" }, { status: 400 });
    }

    const message = await client.messages.create({
      model: "claude-haiku-4-5-20251001",
      max_tokens: 400,
      messages: [
        {
          role: "user",
          content: `You are a pricing engine for an AI agent task marketplace on Solana.

Historical completed tasks:
${JSON.stringify(HISTORICAL_TASKS, null, 2)}

New task to price: "${description.trim()}"

Compare with historical tasks. Respond with ONLY valid JSON, no markdown:
{
  "price_sol": <number 0.01–10.0>,
  "advance_bps": <integer 1000–3000, e.g. 2000 means 20% paid upfront>,
  "complexity": <"low"|"medium"|"high">,
  "reasoning": <one concise sentence>,
  "estimated_duration": <e.g. "2-4h" or "1-2d">
}`,
        },
      ],
    });

    const text = (message.content[0] as { text: string }).text.trim();
    const result = JSON.parse(text);

    // Find 3 most relevant similar tasks
    const similar = HISTORICAL_TASKS.filter(
      (t) => t.complexity === result.complexity
    ).slice(0, 3);

    return Response.json({ ...result, similar });
  } catch (e) {
    console.error("estimate-price error:", e);
    return Response.json({ error: "Estimation failed" }, { status: 500 });
  }
}
