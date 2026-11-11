<p align="center"><img src="assets/logo.svg" alt="thrift" width="300"></p>

Routes each Claude request to the cheapest tier (Haiku, Sonnet or Opus) that can handle it. A local Ollama model classifies the prompt, then a LiteLLM proxy forwards it. Routing never leaves the machine.

```bash
uv venv --python 3.12 && uv pip install -e .
thrift init              # all-local preset (writes .env, runs doctor)
thrift init --bedrock    # or the Bedrock preset: then fill in AWS_PROFILE and THRIFT_* model IDs
thrift doctor            # re-check the setup, prints what to fix
thrift serve             # http://127.0.0.1:4000
eval "$(thrift env)"     # in the shell where you run Claude Code
```

If the classifier is down or returns garbage, requests fall back to the moderate (Sonnet) tier. Any OpenAI- or Anthropic-compatible client works with `model="auto"`.

Optional: `build_labels.py` labels your Claude Code transcripts by effort into `labels.csv` (gitignored) for `thrift bench`.

## Router model

`thrift bench <model>` on `labels.csv` (n=182 prompts with state >= 120 chars; label proxy is tool-call count). Warm model, local Ollama.

| model | acc | within1 | p50 | p95 |
|---|---|---|---|---|
| qwen3.6:35b-a3b (default) | 0.423 | 0.808 | 0.54s | 1.05s |
| qwen3:4b | 0.374 | 0.698 | 0.40s | 0.97s |

Both meet p50 <= 1.0s, so the default is the best within1. Override with `THRIFT_ROUTER_MODEL`. Only models already pulled were tested.
