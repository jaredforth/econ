"""Laya picks the tier in one ~33ms forward pass; the cheapest model that fits answers.

    .venv/bin/python router.py "your prompt"        # route + answer (needs ANTHROPIC_API_KEY)
    .venv/bin/python router.py --dry "your prompt"  # route only
"""
import sys

import laya

# ponytail: 3 static tiers, fallback to the middle on abstain. Add tiers/providers when needed.
MODELS = {"simple": "claude-haiku-5-5", "moderate": "claude-sonnet-5-5", "hard": "claude-opus-5-5"}
QUESTIONS = {"tier": {"type": "choice", "instructions": "How much reasoning does answering this request need?",
    "criteria": {
        "simple": "lookup, rewrite, summarize, classify, extract, short factual or formatting task",
        "moderate": "ordinary coding, analysis, or writing with a few steps",
        "hard": "multi-step reasoning, architecture, hard debugging, math, or high-stakes judgment"}}}

_agent = None


def route(prompt: str) -> str:
    global _agent
    _agent = _agent or laya.Agent("convaiinnovations/laya-multilingual")
    a = _agent.predict(prompt, QUESTIONS)["answers"]["tier"]
    # abstain / not confident -> don't gamble on cheap; use the middle tier
    tier = "moderate" if a.get("abstain") or len(a.get("prediction_set", [])) > 1 else a["choice"]
    return MODELS[tier]


if __name__ == "__main__":
    dry = "--dry" in sys.argv
    prompt = " ".join(x for x in sys.argv[1:] if x != "--dry")
    model = route(prompt)
    print(f"[{model}]", file=sys.stderr)
    if not dry:
        import anthropic
        r = anthropic.Anthropic().messages.create(model=model, max_tokens=2048,
                                                  messages=[{"role": "user", "content": prompt}])
        print(r.content[0].text)
