import argparse
import asyncio
import os
import secrets
import sys

import httpx

from thrift import classify
from thrift.classify import pick_tier


def route(args):
    text = sys.stdin.read() if args.prompt == "-" else args.prompt
    tier = asyncio.run(pick_tier(text))
    print(tier)
    model = os.environ.get(f"THRIFT_{tier.upper()}")
    if model:
        print(model)


def serve(args):
    from dotenv import load_dotenv

    load_dotenv(".env")
    cfg = args.config
    if not cfg:
        home = os.environ.get("THRIFT_HOME")
        cands = ["config.yaml"] + ([os.path.join(home, "config.yaml")] if home else [])
        cfg = next((c for c in cands if os.path.exists(c)), None)
    if not cfg or not os.path.exists(cfg):
        print("thrift: config.yaml not found (run `thrift init` or set THRIFT_HOME)", file=sys.stderr)
        sys.exit(2)
    if not os.environ.get("LITELLM_MASTER_KEY"):
        print("thrift: warning: LITELLM_MASTER_KEY is not set", file=sys.stderr)
    # hook.py lives next to config.yaml; make it importable from any cwd
    cfg_dir = os.path.dirname(os.path.abspath(cfg))
    os.environ["PYTHONPATH"] = os.pathsep.join(filter(None, [cfg_dir, os.environ.get("PYTHONPATH")]))
    argv = [sys.executable, "-m", "litellm.proxy.proxy_cli", "--config", cfg, "--host", args.host, "--port", str(args.port)]
    os.execvp(argv[0], argv)


def _ollama_models():
    r = httpx.get(classify.ollama_host() + "/api/tags", timeout=2)
    r.raise_for_status()
    return [m["name"] for m in r.json().get("models", [])]


def check_ollama():
    try:
        _ollama_models()
        return True, ""
    except Exception:
        return False, "ollama serve"


def check_router_model():
    m = classify.ROUTER_MODEL
    fix = f"ollama pull {m}"
    try:
        names = _ollama_models()
    except Exception:
        return False, fix + "  (after Ollama is running)"
    return (m in names or f"{m}:latest" in names), fix


def check_env_file():
    return os.path.exists(".env"), "create .env here (see README) or run from your thrift dir"


def _env_check(name, hint):
    return lambda: (bool(os.environ.get(name)), f"set {name}={hint} in .env")


def check_aws():
    tiers = [os.environ.get(f"THRIFT_{t.upper()}", "") for t in classify.TIERS]
    if not any(t.startswith("bedrock/") for t in tiers):
        return True, ""
    ok = bool(os.environ.get("AWS_PROFILE") or os.environ.get("AWS_ACCESS_KEY_ID"))
    return ok, "set AWS_PROFILE=<profile> (or AWS_ACCESS_KEY_ID/AWS_SECRET_ACCESS_KEY) in .env"


CHECKS = [
    ("Ollama reachable", check_ollama),
    ("Router model pulled", check_router_model),
    (".env found", check_env_file),
    ("LITELLM_MASTER_KEY set", _env_check("LITELLM_MASTER_KEY", "sk-<anything>")),
    *[(f"THRIFT_{t.upper()} set", _env_check(f"THRIFT_{t.upper()}", "<litellm model string>")) for t in classify.TIERS],
    ("AWS creds (bedrock tiers)", check_aws),
]


def doctor(args):
    from dotenv import load_dotenv

    load_dotenv(".env")
    bad = 0
    for name, fn in CHECKS:
        ok, fix = fn()
        print(f"PASS  {name}" if ok else f"FAIL  {name}\n      fix: {fix}")
        bad += not ok
    if bad:
        sys.exit(1)


def init(args):
    if os.path.exists(".env"):
        print("`.env` exists, leaving it")
    else:
        head, *blocks = open(".env.example").read().split("# --- ")
        want = "Bedrock" if args.bedrock else "All local"
        block = next(b for b in blocks if b.startswith(want))
        body = "\n".join(l[2:] if l.startswith("# ") else l for l in block.splitlines()[1:])
        key = "sk-" + secrets.token_hex(16)
        head = head.replace("sk-change-me", key)
        open(".env", "w").write(head + body.strip() + "\n")
        print("wrote .env")
        if args.bedrock:
            print("fill in AWS_PROFILE and the bedrock/... THRIFT_* model IDs in .env")
    try:
        doctor(args)
    except SystemExit:
        pass
    print("next: thrift serve")


def env(args):
    from dotenv import load_dotenv

    load_dotenv(".env")
    key = os.environ.get("LITELLM_MASTER_KEY")
    if not key:
        print("thrift: LITELLM_MASTER_KEY is not set (run `thrift init`)", file=sys.stderr)
        sys.exit(2)
    print(f"export ANTHROPIC_BASE_URL=http://127.0.0.1:{args.port}")
    print(f"export ANTHROPIC_AUTH_TOKEN={key}")
    print("export ANTHROPIC_MODEL=auto")


def bench(args):
    if not os.path.exists("labels.csv"):
        print("run build_labels.py first", file=sys.stderr)
        sys.exit(2)
    from thrift.bench import run

    run(args.models)


def main(argv=None):
    p = argparse.ArgumentParser(prog="thrift")
    sub = p.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("route", help="print the tier (and model) for a prompt")
    r.add_argument("prompt", help="prompt text, or - to read stdin")
    r.set_defaults(fn=route)
    v = sub.add_parser("serve", help="run the LiteLLM router proxy")
    v.add_argument("--port", type=int, default=4000)
    v.add_argument("--host", default="127.0.0.1")
    v.add_argument("--config")
    v.set_defaults(fn=serve)
    d = sub.add_parser("doctor", help="check the setup and say what to fix")
    d.set_defaults(fn=doctor)
    i = sub.add_parser("init", help="create .env and run doctor")
    g = i.add_mutually_exclusive_group()
    g.add_argument("--local", action="store_true", help="all-local Ollama preset (default)")
    g.add_argument("--bedrock", action="store_true", help="Bedrock preset")
    i.set_defaults(fn=init)
    e = sub.add_parser("env", help='print exports for Claude Code: eval "$(thrift env)"')
    e.add_argument("--port", type=int, default=4000)
    e.set_defaults(fn=env)
    b = sub.add_parser("bench", help="score router models on labels.csv")
    b.add_argument("models", nargs="+")
    b.set_defaults(fn=bench)
    args = p.parse_args(argv)
    args.fn(args)
    return 0


if __name__ == "__main__":
    main()
