<p align="center"><img src="assets/icon.png" width="128" alt="thrift logo"></p>

```console
$ thrift "rename this variable across the file"
simple
```

A small model reads your prompt and picks the cheapest tier that can handle it: `simple`, `moderate` or `hard`. The router is either a local [Ollama](https://ollama.com) model or Claude Haiku. If the router is down or returns garbage, you get `moderate`.

```bash
cargo install --path .
```

## Config

```bash
thrift init            # writes the config with defaults (Ollama router, haiku/sonnet/opus tiers)
thrift init haiku      # same, with Claude Haiku as the router
```

`init` only fills in what's missing and keeps your other keys, so it's safe to re-run. Settings live in `~/.config/thrift/config.json` (or `$XDG_CONFIG_HOME/thrift/config.json`, or the path in `$THRIFT_CONFIG`): a flat JSON object of strings. Environment variables with the same names override the file. A missing file is fine; a malformed one is an error.

```json
{
  "THRIFT_ROUTER_MODEL": "haiku",
  "ANTHROPIC_API_KEY": "sk-ant-...",
  "THRIFT_SIMPLE": "haiku",
  "THRIFT_MODERATE": "sonnet",
  "THRIFT_HARD": "opus"
}
```

| key | default | |
|---|---|---|
| `THRIFT_ROUTER_MODEL` | `qwen3:4b` | Ollama model, `haiku`, or a `claude-*` model id |
| `OLLAMA_HOST` | `http://localhost:11434` | Ollama only |
| `ANTHROPIC_API_KEY` | unset | required when the router is Claude |
| `ANTHROPIC_BASE_URL` | `https://api.anthropic.com` | Claude only |
| `THRIFT_SIMPLE` / `THRIFT_MODERATE` / `THRIFT_HARD` | unset | model to print for each tier; unset prints the tier name |

```bash
claude --model "$(thrift 'fix the typo in README')"
```

## Test

```bash
cargo llvm-cov --fail-uncovered-lines 0
```
