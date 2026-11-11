<p align="center"><img src="assets/logo.svg" alt="thrift" width="300"></p>

Routes each prompt to the cheapest Claude tier that can handle it. [Laya](https://github.com/NandhaKishorM/laya) classifies the prompt locally in one forward pass (PyTorch, no Ollama); Haiku, Sonnet or Opus answers over the API.

```bash
uv venv --python 3.12 && uv pip install laya anthropic
.venv/bin/python router.py --dry "your prompt"   # route only
.venv/bin/python router.py "your prompt"         # route and answer (ANTHROPIC_API_KEY)
```

`build_labels.py` labels your Claude Code transcripts by effort, and `fit.py` fits Laya's tier head on them. Both stay local: `labels.csv` and `fit-tier/` are gitignored.
