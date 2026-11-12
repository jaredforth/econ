<p align="center"><img src="assets/icon.png" width="128" alt="econ logo"></p>

```console
$ econ rename this variable across the file
simple
$ git diff | econ --tier
moderate
```

A small model reads your prompt and picks the cheapest tier that can handle it: `simple`, `moderate` or `hard`. The router is either a local [Ollama](https://ollama.com) model or Claude Haiku. If the router is down or returns garbage, you get `moderate`.

```bash
cargo install --path .
econ --help
```

## Config

```bash
econ init            # writes the config with defaults (Ollama router, haiku/sonnet/opus tiers)
econ init haiku      # same, with Claude Haiku as the router
```

`init` only fills in what's missing and keeps your other keys, so it's safe to re-run. Settings live in `~/.config/econ/config.json` (or `$XDG_CONFIG_HOME/econ/config.json`, or the path in `$ECON_CONFIG`): a flat JSON object of strings. Environment variables with the same names override the file. A missing file is fine; a malformed one is an error.

```json
{
  "ECON_ROUTER_MODEL": "haiku",
  "ANTHROPIC_API_KEY": "sk-ant-...",
  "ECON_SIMPLE": "haiku",
  "ECON_MODERATE": "sonnet",
  "ECON_HARD": "opus"
}
```

| key | default | |
|---|---|---|
| `ECON_ROUTER_MODEL` | `qwen3:4b` | Ollama model, `haiku`, or a `claude-*` model id |
| `OLLAMA_HOST` | `http://localhost:11434` | Ollama only |
| `ANTHROPIC_API_KEY` | unset | required when the router is Claude |
| `ANTHROPIC_BASE_URL` | `https://api.anthropic.com` | Claude only |
| `ECON_SIMPLE` / `ECON_MODERATE` / `ECON_HARD` | unset | model to print for each tier; unset prints the tier name |

```bash
claude --model "$(econ 'fix the typo in README')"
```

## Test

```bash
cargo llvm-cov --fail-uncovered-lines 0
```
