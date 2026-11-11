#!/bin/sh
# Local router on :4000. Needs AWS creds (AWS_PROFILE in .env) and `ollama serve` running.
cd "$(dirname "$0")" && set -a && . ./.env && set +a && exec .venv/bin/litellm --config config.yaml --host 127.0.0.1 --port 4000
