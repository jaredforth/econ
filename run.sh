#!/bin/sh
# Local router on :4000. Needs ANTHROPIC_API_KEY and `ollama serve` running.
cd "$(dirname "$0")" && set -a && . ./.env && set +a && exec .venv/bin/litellm --config config.yaml --host 127.0.0.1 --port 4000
