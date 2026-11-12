<p align="center"><img src="assets/icon.png" width="128" alt="econ logo"></p>

<p align="center">
<a href="https://crates.io/crates/econ"><img src="https://img.shields.io/crates/v/econ.svg" alt="crates.io"></a>
<a href="https://github.com/jaredforth/econ/actions/workflows/ci.yml"><img src="https://github.com/jaredforth/econ/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
<a href="#license"><img src="https://img.shields.io/crates/l/econ.svg" alt="license"></a>
<a href="https://crates.io/crates/econ"><img src="https://img.shields.io/crates/d/econ.svg" alt="downloads"></a>
</p>

```console
$ econ fix the typo in the README     # opens Claude Code on haiku
$ econ design a multi-tenant billing schema   # opens Claude Code on opus
$ econ -n rename this variable        # dry run: just prints the model
haiku
$ git diff | econ --tier              # piped input prints the tier
moderate
```

A small model reads your prompt and picks the cheapest tier that can handle it: `simple`, `moderate` or `hard`. Then `econ` opens Claude Code (`claude --model <model> "<prompt>"`) on that tier's model: `haiku`, `sonnet` or `opus` unless you map them differently. The router is either a local [Ollama](https://ollama.com) model or Claude Haiku. If the router is down or returns garbage, you get `moderate`.

## Install

Needs [Rust](https://rustup.rs) and [Claude Code](https://claude.com/claude-code) (`claude` on your `PATH`).

```bash
cargo install econ        # from crates.io; re-run to update
econ --help
```

From source: `git clone https://github.com/jaredforth/econ && cargo install --path econ`.

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

To use another command than `claude`, set `ECON_CLAUDE`.

## Benchmarks

36 hand-labeled prompts (12 per tier, in [`bench/prompts.tsv`](bench/prompts.tsv)), routed with the default local router (`qwen3:4b` on Ollama, Apple M3 Pro, model already loaded). Reproduce with `bench/run.sh`.

| | result |
|---|---|
| Accuracy | 31/36 (86%) |
| `simple` | 12/12 |
| `moderate` | 7/12 |
| `hard` | 12/12 |
| Routed too cheap | 1 |
| Routed too expensive | 4 |
| Latency (`econ --tier`) | median 0.30s, p95 0.34s |

Misses lean toward the pricier tier, so you pay a bit more rather than get a weak answer. Most misses are `moderate` prompts. The labels are the author's judgment and the set is small, so treat this as a sanity check rather than a leaderboard.

## Test

```bash
cargo llvm-cov --fail-uncovered-lines 0
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
