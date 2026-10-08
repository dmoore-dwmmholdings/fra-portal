"""Patch the `event` facts from a batch's input files onto the stored analyses (patch_analysis over HTTP).

The batch input files carry the facts as an `## event` JSON line. Running this after a batch makes the facts
independent of whether the agents' MCP client knows the `event` field.

usage (repo root, FRA_MCP_TOKEN set):  python tools/batch/patch_events.py data/reports/batches/<batch id>
"""
import json
import os
import sys
import urllib.request
from pathlib import Path

URL = "https://fra-portal.web.app/mcp"


def call(token: str, args: dict, n: int) -> str:
    body = json.dumps({"jsonrpc": "2.0", "id": n, "method": "tools/call",
                       "params": {"name": "patch_analysis", "arguments": args}}).encode()
    req = urllib.request.Request(URL, body, {"Authorization": f"Bearer {token}", "Content-Type": "application/json",
                                             "Accept": "application/json, text/event-stream"})
    with urllib.request.urlopen(req, timeout=60) as r:
        res = json.loads(r.read())
    return res.get("result", {}).get("content", [{}])[0].get("text", json.dumps(res))


def main():
    batch = Path(sys.argv[1])
    token = os.environ["FRA_MCP_TOKEN"]
    patched, missing = 0, []
    for n, f in enumerate(sorted(batch.glob("*_*.txt")), 1):
        lines = f.read_text(encoding="utf-8").splitlines()
        event = json.loads(lines[lines.index("## event (copy into put_analysis.event as-is)") + 1])
        ev_id, key = f.stem.rsplit("_", 1)
        out = call(token, {"ev_id": ev_id, "aircraft_key": int(key), "event": event}, n)
        if '"patched"' in out:
            patched += 1
        else:
            missing.append(ev_id)
    print(json.dumps({"patched": patched, "not_stored": missing}))


if __name__ == "__main__":
    main()
