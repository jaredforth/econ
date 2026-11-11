"""LiteLLM pre-call hook: a local Ollama model picks the tier, the request's model is rewritten.
Send requests with model="auto". The prompt never leaves the machine for routing."""
from litellm.integrations.custom_logger import CustomLogger
from thrift.classify import last_user_text, pick_tier


class Router(CustomLogger):
    async def async_pre_call_hook(self, user_api_key_dict, cache, data, call_type):
        if data.get("model") == "auto":
            # ponytail: classifies every request, so a long session can flip tiers; add per-session stickiness if it bites
            data["model"] = await pick_tier(last_user_text(data.get("messages", [])))
        return data


router = Router()
