"""Grant the Cloud Build builder role to a project's default compute service account.

New Firebase projects can lack this grant, which makes Cloud Functions builds fail with
"missing permission on the build service account". Uses the Firebase CLI's stored login.

Usage: python tools/firebase/grant_build_role.py <project-id>
"""

import json
import os
import sys
import urllib.request

ROLE = "roles/cloudbuild.builds.builder"


def call(method, url, token, body=None):
    req = urllib.request.Request(
        url,
        method=method,
        data=json.dumps(body).encode() if body is not None else None,
        headers={"Authorization": f"Bearer {token}", "Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req) as resp:
        return json.load(resp)


def main():
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    project = sys.argv[1]
    cfg = os.path.expanduser("~/.config/configstore/firebase-tools.json")
    token = json.load(open(cfg))["tokens"]["access_token"]

    crm = f"https://cloudresourcemanager.googleapis.com/v1/projects/{project}"
    number = call("GET", crm, token)["projectNumber"]
    member = f"serviceAccount:{number}-compute@developer.gserviceaccount.com"

    policy = call("POST", f"{crm}:getIamPolicy", token, {"options": {"requestedPolicyVersion": 3}})
    binding = next((b for b in policy.setdefault("bindings", []) if b["role"] == ROLE), None)
    if binding and member in binding["members"]:
        print(f"{member} already has {ROLE}")
        return
    if binding:
        binding["members"].append(member)
    else:
        policy["bindings"].append({"role": ROLE, "members": [member]})

    call("POST", f"{crm}:setIamPolicy", token, {"policy": policy})
    print(f"granted {ROLE} to {member}")


if __name__ == "__main__":
    main()
