"""Transcripts -> labels.csv. Label = effort the prompt actually took (tool calls until the next
human prompt). ponytail: effort is a proxy for "needed a bigger model"; swap in pass/fail per model if you log it."""
import csv, glob, json, os, random

def text(m):
    c = m.get("content")
    if isinstance(c, str): return c
    if any(b.get("type") == "tool_result" for b in c): return None
    return " ".join(b.get("text", "") for b in c if b.get("type") == "text")

rows = []
for f in glob.glob(os.path.expanduser("~/.claude/projects/*/*.jsonl")):
    cur = None
    for l in open(f):
        try: d = json.loads(l)
        except ValueError: continue
        if d.get("isSidechain"): continue
        m = d.get("message", {})
        if d.get("type") == "user":
            t = text(m)
            if t is None: continue
            if cur: rows.append(cur)
            t = t.strip()
            cur = {"state": t[:1500], "n": 0} if t and not t.startswith(("<", "/", "[Request", "Caveat")) and len(t) > 15 else None
        elif d.get("type") == "assistant" and cur:
            cur["n"] += sum(b.get("type") == "tool_use" for b in m.get("content", []) if isinstance(b, dict))
    if cur: rows.append(cur)

for r in rows: r["tier"] = "simple" if r["n"] <= 2 else "moderate" if r["n"] <= 12 else "hard"
random.seed(0); random.shuffle(rows)
by = {t: [r for r in rows if r["tier"] == t][:200] for t in ("simple", "moderate", "hard")}
out = [r for v in by.values() for r in v]; random.shuffle(out)
with open("labels.csv", "w", newline="") as fh:
    w = csv.writer(fh); w.writerow(["state", "tier"]); w.writerows((r["state"], r["tier"]) for r in out)
print({t: len(v) for t, v in by.items()}, "of", len(rows), "prompts")
