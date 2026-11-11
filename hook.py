"""LiteLLM pre-call hook: a local Ollama model picks the tier, the request's model is rewritten.
Send requests with model="auto". The prompt never leaves the machine for routing."""
import httpx
from litellm.integrations.custom_logger import CustomLogger

OLLAMA = "http://localhost:11434/api/chat"
ROUTER_MODEL = "qwen3.6:35b-a3b"  # any local Ollama model; a 3-4B one is faster
SYSTEM = ("Classify how much reasoning the user's request needs. "
          "simple: lookup, rewrite, summarize, extract, short factual or formatting task. "
          "moderate: ordinary coding, analysis or writing with a few steps. "
          "hard: multi-step reasoning, architecture, hard debugging, math, high-stakes judgment. "
          "Answer with the tier only.")


def last_user_text(messages):
    for m in reversed(messages):
        if m.get("role") != "user":
            continue
        c = m.get("content")
        if isinstance(c, list):
            c = " ".join(b.get("text", "") for b in c if b.get("type") == "text")
        if c and c.strip():
            return c
    return ""


async def pick_tier(text):
    body = {"model": ROUTER_MODEL, "stream": False, "think": False,
            "options": {"temperature": 0, "num_predict": 20},
            "format": {"type": "object", "properties": {"tier": {"enum": ["simple", "moderate", "hard"]}},
                       "required": ["tier"]},
            "messages": [{"role": "system", "content": SYSTEM}, {"role": "user", "content": text[:4000]}]}
    try:
        async with httpx.AsyncClient(timeout=30) as c:
            r = await c.post(OLLAMA, json=body)
        return __import__("json").loads(r.json()["message"]["content"])["tier"]
    except Exception:
        return "moderate"  # router down or garbled: don't gamble on cheap


class Router(CustomLogger):
    async def async_pre_call_hook(self, user_api_key_dict, cache, data, call_type):
        if data.get("model") == "auto":
            # ponytail: classifies every request, so a long session can flip tiers; add per-session stickiness if it bites
            data["model"] = await pick_tier(last_user_text(data.get("messages", [])))
        return data


router = Router()
