#!/usr/bin/env bash
#
# Reports the documentation a change to the public API has left behind.
#
# `make check` proves the code is correct, and `cargo test --doc` proves
# the examples in doc comments still compile and produce what they claim.
# Neither can tell that a section of README.md still describes a
# signature that changed, or that AGENTS.md names an invariant that no
# longer holds. Those are prose; nothing compiles them. This is the
# residue, and this script names it.
#
# It is a reminder, not a gate: it always exits 0. A diff can legitimately
# touch a public item without any of these needing an edit - a doc comment
# fix, for instance - and a check that cried wolf would be turned off
# within a week.
#
# Compares against HEAD by default, or against the ref given as $1.
set -euo pipefail

cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

base="${1:-HEAD}"

# No commits yet: nothing to diff against.
git rev-parse --verify --quiet "$base" >/dev/null 2>&1 || exit 0

changed=$(git diff --name-only "$base" -- '*.rs' ':!tests/*' ':!**/tests/*')

if [ -z "$changed" ]; then
    exit 0
fi

# A line added or removed at the start of a public declaration. Deliberately
# a heuristic over the diff rather than a real API diff (which would mean
# building rustdoc JSON on two worktrees) - the point is to prompt a human,
# not to be exhaustive.
surface_changed=$(
    git diff -U0 "$base" -- '*.rs' ':!tests/*' ':!**/tests/*' |
        grep -E '^[+-]\s*pub (fn|struct|enum|trait|type|const|static|mod|use) ' |
        head -1 ||
        true
)

# The CLI's flags are public API too (see "API stability" in README.md),
# and they are string literals rather than `pub` items, so the pattern
# above cannot see them. A changed "--flag" in the binary counts.
if [ -z "$surface_changed" ]; then
    surface_changed=$(
        git diff -U0 "$base" -- 'src/bin/*.rs' |
            grep -E '^[+-].*"--[a-z][a-z-]*"' |
            head -1 ||
            true
    )
fi

if [ -z "$surface_changed" ]; then
    exit 0
fi

# Everything worth checking by hand when the public API moves, plus the
# reason each one cannot be checked automatically.
declare -a targets=(
    "README.md|the prose around Usage / Regional variants / Gender, which no doctest reads"
    "README.pt-BR.md|same, and it has to stay in sync with README.md"
    "AGENTS.md|if an invariant changed (CLAUDE.md just imports it)"
    "CHANGELOG.md|every public change belongs here"
    "skills/using-numbertext/SKILL.md|it documents the same public API for consumer projects"
)

stale=()

for entry in "${targets[@]}"; do
    path="${entry%%|*}"
    reason="${entry#*|}"

    if ! git diff --quiet "$base" -- "$path" 2>/dev/null; then
        continue
    fi

    stale+=("  $path
      $reason")
done

if [ ${#stale[@]} -eq 0 ]; then
    exit 0
fi

printf '\nThis diff changes the public API. Untouched documentation:\n\n'
printf '%s\n' "${stale[@]}"
printf '\nIf none of the above needs an edit, nothing to do.\n\n'
