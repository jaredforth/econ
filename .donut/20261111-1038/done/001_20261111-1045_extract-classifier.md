# Extract the tier classifier into a reusable module

The Ollama classifier lives inside `hook.py`, which imports LiteLLM, so nothing else (a CLI, tests) can use it cheaply. Move it into a plain module.

**Slug:** extract-classifier
**Depends on:** None
**Verify:** `.venv/bin/python -m unittest tests.test_classify -q`

## Goal
`thrift/classify.py` holds `last_user_text` and `pick_tier` with no LiteLLM import. `hook.py` and `bench.py` import from it. Behavior is unchanged except the Ollama URL and model become env-configurable.

## Context
- `hook.py` today defines `OLLAMA`, `ROUTER_MODEL = "qwen3.6:35b-a3b"`, `SYSTEM`, `last_user_text(messages)`, `async pick_tier(text)` (returns "simple"|"moderate"|"hard", falls back to "moderate" on any exception) and the `Router(CustomLogger)` class exported as `router` (referenced by `config.yaml` as `callbacks: hook.router`).
- `bench.py` sets `hook.ROUTER_MODEL = m` then calls `hook.pick_tier`. It must keep working.
- No `tests/` dir and no pytest in the venv: use stdlib `unittest`.

## Preflight
- `grep -n "def pick_tier\|def last_user_text\|ROUTER_MODEL" hook.py bench.py` shows the anchors above.
- `ls thrift tests` must fail (neither exists).

## Files
- Create: `thrift/__init__.py` (empty), `thrift/classify.py`, `tests/__init__.py` (empty), `tests/test_classify.py`
- Modify: `hook.py` (delete moved code, `from thrift.classify import last_user_text, pick_tier`), `bench.py` (set `thrift.classify.ROUTER_MODEL`, call `thrift.classify.pick_tier`)
- Do not touch: `config.yaml`, `router.py`, `fit.py`

## Requirements
1. `thrift.classify.pick_tier(text, model=None)` is async; model defaults to module-level `ROUTER_MODEL = os.environ.get("THRIFT_ROUTER_MODEL", "qwen3.6:35b-a3b")` (read at import; the default is changed by a later ticket).
2. The Ollama chat URL is `os.environ.get("OLLAMA_HOST", "http://localhost:11434").rstrip("/") + "/api/chat"`; if the host has no scheme, prefix `http://`.
3. Any exception or an answer outside the three tiers returns `"moderate"`.
4. `hook.router` still works: `config.yaml` is unchanged.

## Implementation steps
Cut/paste the functions and `SYSTEM` into `thrift/classify.py` and `import json` at top (replace the `__import__("json")` hack). Validate the returned tier with `if tier not in TIERS: return "moderate"`. Keep `TIERS = ("simple","moderate","hard")` exported.

## Testing
`tests/test_classify.py` (unittest): (a) `last_user_text` returns the last user string, joins text blocks of a list content, skips a user turn that only has `tool_result` blocks and returns the earlier one, returns "" for empty; (b) `asyncio.run(pick_tier("x"))` with `OLLAMA_HOST=http://127.0.0.1:9` (closed port) returns `"moderate"`.
Expected: `Ran 2+ tests ... OK`. Also `.venv/bin/python -c "import hook; print(hook.router)"` imports cleanly.

## Acceptance criteria
- [x] Tests pass; `hook.py` no longer defines the classifier
- [x] `.venv/bin/python -m py_compile bench.py hook.py` passes (do not import `bench`; it runs on import)

## Out of scope
Changing the default model, the prompt, CLI work.

## Completion notes
- Changes: added `thrift/__init__.py`, `thrift/classify.py` (TIERS, ROUTER_MODEL from THRIFT_ROUTER_MODEL, SYSTEM, last_user_text, pick_tier(text, model=None), OLLAMA_HOST handling with scheme prefix); `tests/__init__.py`, `tests/test_classify.py`; `hook.py` now imports from `thrift.classify`; `bench.py` imports `thrift.classify as classify` and sets `classify.ROUTER_MODEL`.
- Tests: `.venv/bin/python -m unittest tests.test_classify -q` -> `Ran 5 tests in 0.063s` / `OK`. `.venv/bin/python -c "import hook; print(hook.router)"` -> `<hook.Router object at 0x104ca1700>`. `py_compile bench.py hook.py` passed.
- Deviations: none.
- Open issues: none.
