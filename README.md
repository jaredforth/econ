<p align="center"><img src="assets/logo.svg" alt="thrift" width="300"></p>

Routes each prompt to the cheapest Claude tier that can handle it. [Laya](https://github.com/NandhaKishorM/laya) classifies the prompt locally in one forward pass (PyTorch, no Ollama); Haiku, Sonnet or Opus answers over the API.

```bash
uv venv --python 3.12 && uv pip install laya anthropic
.venv/bin/python router.py --dry "your prompt"   # route only
.venv/bin/python router.py "your prompt"         # route and answer (ANTHROPIC_API_KEY)
```

`build_labels.py` labels your Claude Code transcripts by effort, and `fit.py` fits Laya's tier head on them. Both stay local: `labels.csv` and `fit-tier/` are gitignored.

## Local router proxy

A local Ollama model classifies each request, then LiteLLM forwards it to Haiku, Sonnet or Opus. Routing never leaves the machine.

```bash
ollama serve &                      # needs a local model; set ROUTER_MODEL in hook.py
export ANTHROPIC_API_KEY=...
./run.sh                            # http://127.0.0.1:4000, master key in .env
```

Point any OpenAI- or Anthropic-compatible client at it with `model="auto"`. For Claude Code: `ANTHROPIC_BASE_URL=http://127.0.0.1:4000` and `ANTHROPIC_AUTH_TOKEN=<LITELLM_MASTER_KEY from .env>`. Router down or garbled falls back to Sonnet.
