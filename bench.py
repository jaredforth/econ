"""Score candidate router models on labels.csv: accuracy, within-one-tier, latency. Usage: bench.py model [model...]"""
import asyncio, csv, sys, time, hook
T = ["simple", "moderate", "hard"]
rows = [r for r in csv.DictReader(open("labels.csv", encoding="utf-8")) if len(r["state"]) >= 120]
for m in sys.argv[1:]:
    hook.ROUTER_MODEL = m
    asyncio.run(hook.pick_tier("warm"))
    ok = near = 0; lat = []; conf = {}
    for r in rows:
        t = time.time(); p = asyncio.run(hook.pick_tier(r["state"])); lat.append(time.time() - t)
        ok += p == r["tier"]; near += abs(T.index(p) - T.index(r["tier"])) <= 1
        conf[(r["tier"], p)] = conf.get((r["tier"], p), 0) + 1
    lat.sort()
    print(f"{m}: n={len(rows)} acc={ok/len(rows):.3f} within1={near/len(rows):.3f} p50={lat[len(lat)//2]:.2f}s p95={lat[int(len(lat)*.95)]:.2f}s")
    print("  gold->pred", {f"{g[0]}>{p[0]}": n for (g, p), n in sorted(conf.items())})
