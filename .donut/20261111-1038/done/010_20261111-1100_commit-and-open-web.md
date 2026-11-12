# Commit the finished work and open the repo in the browser

The user asked to commit the work and open the repo on the web. This runs last so the commit holds the whole CLI change.

**Slug:** commit-and-open-web
**Depends on:** `remove-laya-readme`
**Verify:** `test -z "$(git status --porcelain -- . ':!.donut' ':!.claude' ':!.cursor' ':!QWEN.md' ':!skills')"`

## Goal
One commit on `main` with the project changes, then `gh repo view --web` opens https://github.com/jaredforth/thrift in the browser.

## Context
- Remote `origin` is `git@github.com:jaredforth/thrift.git`; branch is `main`. Last commit: `7632c91 Switch proxy to Bedrock, env-driven profile and model IDs`.
- Donut/agent scaffolding is untracked and not part of the project: `.donut/`, `.claude/`, `.cursor/`, `QWEN.md`, `skills/`. Never stage these.
- `.gitignore` already excludes `.venv/`, `__pycache__/`, `labels.csv`, `fit-tier/`, `.env`, `proxy.log`. `fit.log` is NOT ignored: do not commit it unless the ticket for the Laya removal deleted it.
- The user explicitly requested the commit. A push was NOT requested.

## Preflight
- `git status --short` shows only expected files: `pyproject.toml`, `thrift/`, `tests/`, `config.yaml`, `.env.example`, `README.md`, deleted `run.sh`/`router.py`/`fit.py`/`bench.py`. Anything else (secrets, `.env`, large files) -> stop and write `BLOCKED:`.
- `.venv/bin/python -m unittest discover -s tests -q` passes.
- `which gh` succeeds.

## Files
- Modify: none
- Do not touch: any source file, `.env`, the scaffolding paths above

## Requirements
1. Stage by explicit path (no `git add -A`/`.`), including deletions.
2. Single commit, message `Add thrift CLI: local tier router with init, serve, doctor, env, bench`, ending with the attribution lines the session gives.
3. No push, no amend of existing commits, no hook skipping.
4. Run `gh repo view jaredforth/thrift --web`; if the repo does not exist remotely or `gh` is unauthenticated, record that in Open issues instead of creating or pushing anything.

## Testing
Verify command exits 0 after the commit; `git log -1 --stat` lists only project files; `git status --short` shows only the scaffolding paths.

## Acceptance criteria
- [x] Commit exists on `main`, no scaffolding or secrets inside it
- [x] Browser opened (or failure recorded)
- [ ] **Human check:** the user decides whether to `git push`

## Out of scope
Pushing, creating the GitHub repo, tagging, PRs.

## Addendum 1: planner recovery
The spec missed two untracked files. Corrections:
- Preflight: `thrift.egg-info/` and `uv.lock` are expected. Ignore them in the "only expected files" check.
- Append `*.egg-info/` to `.gitignore` (the one allowed edit; `Files: Modify: none` is overridden for `.gitignore` only). Stage and commit `.gitignore` and `uv.lock` with the rest (explicit paths, `uv.lock` is a normal lockfile for a uv-managed project). Never stage `thrift.egg-info/`.
- `fit.log` is tracked but `fit.py` is deleted: leave `fit.log` alone (do not stage or delete it). If the Verify command then reports `fit.log`, it is unmodified and does not appear in `git status`, so no action is needed.
- `gh` is not authenticated and the worker cannot fix that. Do NOT block on it. Still run `gh repo view jaredforth/thrift --web`; on failure record "gh not logged in, user must run `gh auth login`" under Open issues. The ticket is complete once the commit exists and Verify passes.

## Completion notes
- Changes: commit b944cfa on main, "Add thrift CLI: local tier router with init, serve, doctor, env, bench" (14 files: .gitignore +`*.egg-info/`, README.md, thrift/{bench,classify,cli}.py, tests/test_{doctor,env,init,serve}.py, uv.lock, deletions of bench.py/fit.py/router.py/run.sh). No scaffolding, `.env`, `thrift.egg-info/` or `fit.log` committed. Not pushed.
- Tests: `.venv/bin/python -m unittest discover -s tests -q` -> Ran 18 tests, OK. Verify command exits 0 (verify=0). `git status --short` shows only .donut scaffolding.
- Deviations: first `git add` failed on the already-staged `run.sh` (atomic, nothing added), so the initial commit held only the pre-staged run.sh deletion and bench rename. I undid it with `git reset --soft HEAD~1` (local, unpushed, no history lost) and recommitted with the right paths.
- Open issues:
  - `gh repo view jaredforth/thrift --web` failed: gh not logged in, user must run `gh auth login`.
  - Human check: the user decides whether to `git push`.
