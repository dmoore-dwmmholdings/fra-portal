"""Grant IAM roles to a project's default compute service account.

New Firebase projects can lack the default grants for this account. Cloud Functions build and
run as it, so without them builds fail ("missing permission on the build service account") and
Firestore calls fail (PERMISSION_DENIED). Uses the Firebase CLI's stored login.

Usage: python tools/firebase/grant_build_role.py <project-id> [role ...]
Default roles: roles/cloudbuild.builds.builder roles/datastore.user
"""

import json
import os
import sys
import urllib.request

DEFAULT_ROLES = ["roles/cloudbuild.builds.builder", "roles/datastore.user"]


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
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    project, roles = sys.argv[1], sys.argv[2:] or DEFAULT_ROLES
    cfg = os.path.expanduser("~/.config/configstore/firebase-tools.json")
    token = json.load(open(cfg))["tokens"]["access_token"]

    crm = f"https://cloudresourcemanager.googleapis.com/v1/projects/{project}"
    number = call("GET", crm, token)["projectNumber"]
    member = f"serviceAccount:{number}-compute@developer.gserviceaccount.com"

    policy = call("POST", f"{crm}:getIamPolicy", token, {"options": {"requestedPolicyVersion": 3}})
    added = []
    for role in roles:
        binding = next((b for b in policy.setdefault("bindings", []) if b["role"] == role), None)
        if binding and member in binding["members"]:
            print(f"{member} already has {role}")
        elif binding:
            binding["members"].append(member)
            added.append(role)
        else:
            policy["bindings"].append({"role": role, "members": [member]})
            added.append(role)

    if added:
        call("POST", f"{crm}:setIamPolicy", token, {"policy": policy})
        print(f"granted {', '.join(added)} to {member}")


if __name__ == "__main__":
    main()
