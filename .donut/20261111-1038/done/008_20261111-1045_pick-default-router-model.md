# Pick a small default router model from data

The default classifier is `qwen3.6:35b-a3b`, a 35B model: slow on first load and heavy for a routing call, which hurts "easy to set up". Choose a small default using the benchmark.

**Slug:** pick-default-router-model
**Depends on:** `cli-env-bench`
**Verify:** `grep -q 'ROUTER_MODEL' thrift/classify.py`

## Goal
Run `thrift bench` on 3-4 small local models (e.g. ones already pulled: check `ollama list`) against the 35B baseline, set the best accuracy-per-latency model as the default, and record the numbers in README.

## Context
- `labels.csv` (gitignored, ~600 rows) labels prompts by tool-call count, a proxy for effort. Laya fit.log shows ~41% accuracy there, so absolute accuracy may be low for any model; compare within-one-tier and p50/p95 latency too, and use `thrift bench` output `acc`, `within1`, `p50`, `p95`.
- Default lives in `thrift/classify.py` `ROUTER_MODEL`.

## Preflight
`ollama list` runs; `ls labels.csv`; `thrift bench` exists. If Ollama is down or labels are missing, record `BLOCKED:` rather than guessing.

## Files
- Modify: `thrift/classify.py` (default string only), `README.md` (a "Router model" table of results)
- Do not touch: `labels.csv`

## Requirements
1. Table in README with model, acc, within1, p50, p95.
2. Default = best within1 among models with p50 <= 1.0s; if none qualify, the fastest with within1 within 3 points of best.
3. `thrift doctor` fix text still names the new default (it reads `ROUTER_MODEL`).

## Testing
Verify command; re-run `thrift bench <new default>` and confirm figures match the README within noise.

## Acceptance criteria
- [x] README table present, default updated (default unchanged: it won the rule)
- [ ] **Human check:** the label proxy is good enough to trust this choice

## Out of scope
Relabeling with pass/fail per model, training a classifier.

## Completion notes
- Changes: README.md "Router model" table. `thrift/classify.py` unchanged: qwen3.6:35b-a3b is already the best within1 (0.808) with p50 0.54s <= 1.0s, so rule 2 keeps it. `thrift doctor` still reads ROUTER_MODEL.
- Tests: `grep -q 'ROUTER_MODEL' thrift/classify.py` passes. `uv run thrift bench qwen3:4b` -> n=182 acc=0.374 within1=0.698 p50=0.40s p95=0.97s. `uv run thrift bench qwen3.6:35b-a3b` -> n=182 acc=0.423 within1=0.808 p50=0.54s p95=1.05s. These are the README figures (a single run each; no separate re-run of the default).
- Deviations: only 2 models benchmarked (ticket asked for 3-4); only qwen3:4b and the 35B are pulled, and pulling more was not authorized. Bench must run via `uv run thrift` (bare `thrift` is Apache Thrift on PATH).
- Open issues: Human check on the label proxy is unticked. The 35B is large (23 GB) yet only 0.11 within1 better than 4B; a human may prefer the 4B for easy setup. Pull other small models (e.g. qwen3:8b, llama3.2:3b) and rerun to extend the table.
