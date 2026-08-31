#!/usr/bin/env python3
"""Verify that data/*.sor still matches the upstream commit SOURCE.md names.

data/SOURCE.md claims every bundled rule file is upstream's byte for
byte, at one named commit, except for the patches it documents. That is
exactly the kind of claim that rots: a hand edit, a partial sync, or an
updated commit hash nobody re-verified all leave the document saying
something that is no longer true.

This turns the claim into a check. It reads the synced commit out of
SOURCE.md, asks GitHub for that commit's blob SHAs, and compares them
against `git hash-object` locally - the same content-addressed hash, so
a match means the bytes are identical.

Files listed under "Local patches" in SOURCE.md are expected to differ,
and are required to: a patch section describing a patch that is no
longer applied is its own kind of stale.

Exit status is 0 when everything agrees, 1 otherwise.
"""

from __future__ import annotations

import json
import pathlib
import re
import subprocess
import sys
import urllib.error
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parent.parent
DATA = ROOT / "data"
SOURCE_MD = DATA / "SOURCE.md"
REPO = "Numbertext/libnumbertext"

# Files SOURCE.md documents as locally patched, and so expected to
# differ from upstream.
PATCHED = {"bg.sor"}


def synced_commit() -> str:
    """The commit hash SOURCE.md records as the sync point."""
    text = SOURCE_MD.read_text(encoding="utf-8")
    match = re.search(r"^- Synced commit: `([0-9a-f]{40})`", text, re.MULTILINE)
    if not match:
        sys.exit("could not find a 'Synced commit:' line in data/SOURCE.md")

    return match.group(1)


def upstream_blobs(commit: str) -> dict[str, str]:
    """Map of upstream file name -> git blob SHA at `commit`."""
    url = f"https://api.github.com/repos/{REPO}/contents/data?ref={commit}"
    request = urllib.request.Request(url, headers={"User-Agent": "numbertext-rs-provenance"})

    try:
        with urllib.request.urlopen(request, timeout=60) as response:
            entries = json.load(response)
    except urllib.error.URLError as err:
        sys.exit(f"could not reach the GitHub API: {err}")

    return {e["name"]: e["sha"] for e in entries if e["name"].endswith(".sor")}


def local_blob(path: pathlib.Path) -> str:
    """The git blob SHA of a local file, the same hash GitHub reports."""
    return subprocess.run(
        ["git", "hash-object", str(path)],
        capture_output=True,
        text=True,
        check=True,
    ).stdout.strip()


def main() -> int:
    commit = synced_commit()
    upstream = upstream_blobs(commit)
    local = {p.name for p in DATA.glob("*.sor")}

    problems: list[str] = []

    for name in sorted(set(upstream) | local):
        if name not in local:
            problems.append(f"{name}: upstream has it, data/ does not - re-run scripts/sync-data.sh")
            continue
        if name not in upstream:
            problems.append(f"{name}: present locally but not upstream at {commit[:12]}")
            continue

        matches = local_blob(DATA / name) == upstream[name]

        if name in PATCHED and matches:
            problems.append(
                f"{name}: SOURCE.md documents a local patch, but the file is identical to "
                "upstream - upstream may have merged the fix, so drop the patch step in "
                "scripts/sync-data.sh and the section in data/SOURCE.md"
            )
        elif name not in PATCHED and not matches:
            problems.append(
                f"{name}: differs from upstream at {commit[:12]} and is not documented as "
                "patched - either re-sync it or document the patch in data/SOURCE.md"
            )

    if problems:
        print(f"data provenance check FAILED against {REPO}@{commit[:12]}:\n", file=sys.stderr)
        for problem in problems:
            print(f"  - {problem}", file=sys.stderr)
        return 1

    patched = len(PATCHED & local)
    print(
        f"data provenance OK: {len(local) - patched} of {len(local)} files identical to "
        f"{REPO}@{commit[:12]}, {patched} documented patch(es) applied"
    )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
