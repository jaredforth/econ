"""Ollama tier classifier. No LiteLLM import, so a CLI or tests can use it cheaply."""
import json
import os

import httpx

TIERS = ("simple", "moderate", "hard")
ROUTER_MODEL = os.environ.get("THRIFT_ROUTER_MODEL", "qwen3.6:35b-a3b")  # any local Ollama model; a 3-4B one is faster
SYSTEM = ("Classify how much reasoning the user's request needs. "
          "simple: lookup, rewrite, summarize, extract, short factual or formatting task. "
          "moderate: ordinary coding, analysis or writing with a few steps. "
          "hard: multi-step reasoning, architecture, hard debugging, math, high-stakes judgment. "
          "Answer with the tier only.")


def _url():
    host = os.environ.get("OLLAMA_HOST", "http://localhost:11434")
    if "://" not in host:
        host = "http://" + host
    return host.rstrip("/") + "/api/chat"


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


async def pick_tier(text, model=None):
    body = {"model": model or ROUTER_MODEL, "stream": False, "think": False,
            "options": {"temperature": 0, "num_predict": 20},
            "format": {"type": "object", "properties": {"tier": {"enum": list(TIERS)}},
                       "required": ["tier"]},
            "messages": [{"role": "system", "content": SYSTEM}, {"role": "user", "content": text[:4000]}]}
    try:
        async with httpx.AsyncClient(timeout=30) as c:
            r = await c.post(_url(), json=body)
        tier = json.loads(r.json()["message"]["content"])["tier"]
    except Exception:
        return "moderate"  # router down or garbled: don't gamble on cheap
    return tier if tier in TIERS else "moderate"
