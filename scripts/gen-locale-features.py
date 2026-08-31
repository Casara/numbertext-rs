#!/usr/bin/env python3
"""Regenerate the per-language Cargo features from the data/*.sor file list.

Cargo features have to be declared in Cargo.toml - build.rs can read them
but cannot add them - so the one-feature-per-language list is generated
from the same directory listing build.rs walks, and rewritten in place
between the two marker comments below.

Run this (``make gen-features``) after ``scripts/sync-data.sh`` adds or
removes a locale file. It is idempotent: with the data unchanged it
rewrites an identical block.
"""

from __future__ import annotations

import pathlib
import sys

BEGIN = "# BEGIN generated language features -- run `make gen-features` after changing data/*.sor\n"
END = "# END generated language features\n"

ROOT = pathlib.Path(__file__).resolve().parent.parent


def feature_name(code: str) -> str:
    """Map a .sor base name to its Cargo feature name.

    Cargo derives an env var from a feature by uppercasing it and turning
    '-' into '_', so 'hu_Hung' has to become 'lang-hu-hung' for build.rs
    to find it as CARGO_FEATURE_LANG_HU_HUNG.
    """
    return "lang-" + code.lower().replace("_", "-")


def main() -> int:
    codes = sorted(p.stem for p in (ROOT / "data").glob("*.sor"))
    if not codes:
        print("no data/*.sor files found - run scripts/sync-data.sh first", file=sys.stderr)
        return 1

    lines = [
        BEGIN,
        "[features]\n",
        "# The default embeds every bundled locale. Opt out with\n",
        "# `default-features = false` plus one `lang-<code>` feature per\n",
        '# language you actually need - see "Selecting which languages get\n',
        '# embedded" in README.md.\n',
        'default = ["all-languages"]\n',
        "all-languages = []\n",
    ]
    lines += [f"{feature_name(code)} = []\n" for code in codes]
    lines.append(END)

    manifest = ROOT / "Cargo.toml"
    text = manifest.read_text(encoding="utf-8")

    if BEGIN in text and END in text:
        head, rest = text.split(BEGIN, 1)
        _, tail = rest.split(END, 1)
        new = head + "".join(lines) + tail
    else:
        new = text.rstrip("\n") + "\n\n" + "".join(lines)

    manifest.write_text(new, encoding="utf-8")
    print(f"wrote {len(codes)} language features to {manifest.relative_to(ROOT)}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
