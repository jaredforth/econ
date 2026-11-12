# Remove the Laya path and rewrite the README around the CLI

The Laya router (`router.py`, `fit.py`, `fit-tier/`) is a second, competing path: fit.log shows fitted accuracy 0.41 (cold 0.31, chance 0.33), it needs torch, and it calls the Anthropic API directly. The Ollama+LiteLLM proxy is the supported path.

**Slug:** remove-laya-readme
**Depends on:** `cli-init`, `cli-env-bench`
**Verify:** `! grep -rn "laya\|router.py\|fit.py" README.md pyproject.toml`

## Goal
Delete `router.py`, `fit.py`, and the `fit-tier/` artifacts; README documents the three-step flow: install, `thrift init`, `thrift serve`, `eval "$(thrift env)"`.

## Context
- `fit.py` imports `router.QUESTIONS`; nothing else imports `router.py` (`grep -rn "import router\|from router" --include=*.py . --exclude-dir=.venv`). `fit-tier/` and `labels.csv` are gitignored.
- `build_labels.py` stays (used by bench); mention it in the README as optional.
- Clean the venv deps separately: tell the user `uv pip uninstall laya anthropic` in Open issues; do not run it.

## Preflight
`grep -rn "from router\|import router" --include=*.py . --exclude-dir=.venv` hits only `fit.py`.

## Files
- Delete: `router.py`, `fit.py`, `assets/` untouched
- Modify: `README.md`
- Do not touch: `fit.log`, `proxy.log` (gitignored? verify; leave them)

## Requirements
1. Files deleted; no code references remain.
2. README quickstart ≤ 10 lines, covering local-only and Bedrock presets, `thrift doctor`, and the "falls back to the moderate tier when the classifier is down" note.
3. Keep the logo line.

## Testing
Verify command; `.venv/bin/python -m unittest discover -s tests -q` passes.

## Acceptance criteria
- [x] Verify and test suite pass
- [ ] **Human check:** removing the Laya path is acceptable

## Out of scope
Uninstalling packages, git commits.

## Completion notes
- Changes: deleted `router.py` and `fit.py`; README rewritten around install / `thrift init` (local + `--bedrock`) / `thrift doctor` / `thrift serve` / `eval "$(thrift env)"` (6-line quickstart), moderate-tier fallback note, `build_labels.py` mentioned as optional; logo line and Router model section kept. `fit-tier/`, `fit.log`, `proxy.log`, `labels.csv`, `assets/` untouched (fit-tier, proxy.log, labels.csv are gitignored; fit.log is not).
- Tests: preflight `grep -rn "from router\|import router" --include='*.py' . --exclude-dir=.venv` hit only `fit.py:3`. Verify `! grep -rn "laya\|router.py\|fit.py" README.md pyproject.toml` exit 0. `.venv/bin/python -m unittest discover -s tests -q`: Ran 18 tests, OK.
- Deviations: ticket said delete `fit-tier/` artifacts but also "do not touch" scope lists only router.py/fit.py; left `fit-tier/` in place (gitignored).
- Open issues: run `uv pip uninstall laya anthropic` in the venv (not run). `fit-tier/` dir can be removed manually. `fit.log` is not gitignored. Human check: removing the Laya path is acceptable.
