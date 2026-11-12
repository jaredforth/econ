<p align="center"><img src="assets/logo.svg" alt="thrift" width="300"></p>

```console
$ thrift "rename this variable across the file"
simple
```

A small local model (via [Ollama](https://ollama.com)) reads your prompt and picks the cheapest tier that can handle it: `simple`, `moderate` or `hard`. If the router is down or returns garbage, you get `moderate`.

```bash
cargo install --path .
```

## Config (env vars)

| var | default | |
|---|---|---|
| `THRIFT_ROUTER_MODEL` | `qwen3:4b` | any Ollama model |
| `OLLAMA_HOST` | `http://localhost:11434` | |
| `THRIFT_SIMPLE` / `THRIFT_MODERATE` / `THRIFT_HARD` | unset | model to print for each tier; unset prints the tier name |

```bash
THRIFT_SIMPLE=haiku THRIFT_MODERATE=sonnet THRIFT_HARD=opus claude --model "$(thrift 'fix the typo in README')"
```

## Test

```bash
cargo llvm-cov --fail-uncovered-lines 0
```
