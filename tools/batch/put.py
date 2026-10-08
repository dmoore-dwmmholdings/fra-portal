"""Store one analysis: POST a put_analysis JSON file to the portal's MCP endpoint.

Batch agents use this instead of the MCP tool, whose schema a long-running Claude Code session caches and can
hold stale (old ev_id pattern, no `event` field). The server validates every call.

usage (FRA_MCP_TOKEN set):  python tools/batch/put.py <analysis.json>
The file holds the put_analysis arguments: ev_id, aircraft_key, ntsb_no, event_date, outcome, event, aircraft,
summary, nodes, analysis, analyst, method_version. Prints the server's reply; exits 1 on an error.
"""
import json
import os
import re
import sys
import urllib.request

URL = "https://fra-portal.web.app/mcp"


def main():
    args = json.load(open(sys.argv[1], encoding="utf-8"))
    if not re.fullmatch(r"\d{8}X\d{5}|\d{14}", str(args.get("ev_id", ""))):
        sys.exit(f"bad ev_id {args.get('ev_id')!r}: use the ev_id from the input file exactly")
    body = json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
                       "params": {"name": "put_analysis", "arguments": args}}).encode()
    req = urllib.request.Request(URL, body, {"Authorization": f"Bearer {os.environ['FRA_MCP_TOKEN']}",
                                             "Content-Type": "application/json",
                                             "Accept": "application/json, text/event-stream"})
    with urllib.request.urlopen(req, timeout=60) as r:
        res = json.loads(r.read())
    result = res.get("result", {})
    text = result.get("content", [{}])[0].get("text", json.dumps(res))
    print(text)
    if result.get("isError") or "error" in res or ('"created"' not in text and '"replaced"' not in text):
        sys.exit(1)


if __name__ == "__main__":
    main()
