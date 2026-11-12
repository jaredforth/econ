#!/bin/sh
# Routes every prompt in bench/prompts.tsv (expected-tier<TAB>prompt) and prints a summary.
# usage: cargo build --release && bench/run.sh   (Ollama must be running with qwen3:4b pulled)
cd "$(dirname "$0")/.." || exit 1
echo '{}' > "${TMPDIR:-/tmp}/econ-bench.json"
export ECON_CONFIG="${TMPDIR:-/tmp}/econ-bench.json" ECON_ROUTER_MODEL="${ECON_ROUTER_MODEL:-qwen3:4b}"
./target/release/econ --tier warmup >/dev/null   # load the model before timing
python3 - <<'PY'
import subprocess, time, statistics as st
rank = {"simple": 0, "moderate": 1, "hard": 2}
rows = []
for line in open("bench/prompts.tsv"):
    exp, prompt = line.rstrip("\n").split("\t", 1)
    t = time.time()
    got = subprocess.run(["./target/release/econ", "--tier", prompt], capture_output=True, text=True).stdout.strip()
    rows.append((exp, got, time.time() - t))
n = len(rows)
print(f"accuracy {sum(e == g for e, g, _ in rows)}/{n}")
print(f"routed too cheap {sum(rank[g] < rank[e] for e, g, _ in rows)}, too expensive {sum(rank[g] > rank[e] for e, g, _ in rows)}")
for tier in rank:
    r = [e == g for e, g, _ in rows if e == tier]
    print(f"{tier}: {sum(r)}/{len(r)}")
ts = sorted(t for *_, t in rows)
print(f"latency median {st.median(ts):.2f}s, p95 {ts[int(.95 * n) - 1]:.2f}s, max {ts[-1]:.2f}s")
PY
