"""Fit Laya's tier head on labels.csv; report held-out accuracy cold vs fitted; save to fit-tier/."""
import csv, laya
from router import QUESTIONS

rows = list(csv.DictReader(open("labels.csv", encoding="utf-8")))
test, train = rows[:90], rows[90:]
agent = laya.Agent(vegaml=True)

def acc():
    ok = sum(agent.predict(r["state"], QUESTIONS)["answers"]["tier"]["choice"] == r["tier"] for r in test)
    return ok / len(test)

print("cold accuracy   %.3f" % acc(), flush=True)
print(agent.fit([{"state": r["state"], "gold": {"tier": r["tier"]}} for r in train], questions=QUESTIONS))
print("fitted accuracy %.3f" % acc())
agent.save_calibration("fit-tier")
