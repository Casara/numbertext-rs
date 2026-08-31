# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and the versioning follows [Semantic Versioning](https://semver.org/).
While the project is on 0.x, **a minor release may break the public
API** — see the API stability section in the
[README](README.md#api-stability). Raising the MSRV counts as a breaking
change under those rules.

## [0.1.0] - 2026-08-26

Initial release.

### Added

- A from-scratch Rust interpreter for the Soros language, published as
  the standalone [`soros`](crates/soros) crate: pattern/template
  compilation, `== prefix ==` macros, `$(...)`/`$N` recursive calls,
  `[...]` conditional text, `|` boundary modifiers, and the default
  NUMBERTEXT left-zero-deletion rule — built from the official Soros
  language specification and validated against its own canonical
  examples (string reversal, thousands separator, Devanagari digits).
  It is a separate crate rather than a private module so the boundary
  between the generic interpreter and numbertext-specific concepts is
  enforced by Cargo instead of by convention.
- The public `numbertext` API: `Locale`, which resolves and compiles a
  language once and then converts infallibly, plus one-shot free
  functions (`cardinal`, `ordinal`, `ordinal_number`, `year`,
  `currency`, `money`, `help`, `convert`) for single conversions.
  `register_locale` adds a language at runtime from raw `.sor` text;
  `languages` lists what is available.
- All 52 `data/*.sor` locales from upstream
  [libnumbertext](https://github.com/Numbertext/libnumbertext), synced
  byte for byte (commit `a4b0225813b015a0f796754bc6718be20dd9943c`,
  `VERSION` 1.0.11 — see [data/SOURCE.md](data/SOURCE.md)), plus
  `scripts/sync-data.sh` to refresh them and
  `scripts/check-data-provenance.py` to verify the byte-for-byte claim
  against upstream's own blob hashes on every CI run.
- Regional and script variant support (e.g. `"en-GB"`, `"pt-BR"`,
  `"hu-Hung"`): a code with no `.sor` file of its own falls back to its
  base language's file while still activating that code's
  `[:lang-code:]`-tagged lines. Resolution follows upstream's
  `Numbertext::load`, including the `-`/`_` equivalence that lets
  `"hu-Hung"` reach `hu_Hung.sor`, and the Norwegian special case that
  points `"nb"`/`"nn"`/`"nb-NO"`/`"nn-NO"` at `no.sor` — without which
  that file's Nynorsk lines are unreachable by any code.
- `language_variants()`, listing the variant codes the available rule
  files declare, alongside `languages()` for the codes backed by a file.
  The CLI's `--list` prints both, so shell completion offers `pt-BR`
  and `en-GB` and not just `pt` and `en`.
- Feature-based selective embedding (`default-features = false` plus one
  `lang-<code>` per language): the default embeds every bundled
  language, and opting into a subset saves roughly 236 KB of embedded
  rule data. The per-language features are generated from the data
  directory by `scripts/gen-locale-features.py`, and `build.rs` turns
  the selection into the embedded locale table — so adding a language
  stays a data change.
- A regex engine built on [`fancy-regex`](https://docs.rs/fancy-regex),
  which delegates any pattern without a backreference or lookaround to
  the linear-time [`regex`](https://docs.rs/regex) crate and uses a
  step-bounded backtracking engine only for the rest. This is what makes
  `hu.sor`'s grammatical-suffix "database" lookup (`.*:\1:(-[^-:]*).*`)
  work, including for genuinely ambiguous backreference patterns that a
  "single linear candidate plus verification" approach would silently
  get wrong. The bound (`soros::BACKTRACK_LIMIT`) is a step count rather
  than a wall-clock timeout, so behavior is reproducible across
  machines.
- A bounded recursion depth (`soros::MAX_RECURSION_DEPTH`) so a rule file
  that recurses without converging yields an empty string instead of
  overflowing the stack and aborting the process.
- Graceful degradation for any rule that cannot be compiled: recorded in
  `Program::skipped` (surfaced as `Locale::skipped_rules`) rather than
  failing the whole locale. Every bundled locale currently compiles with
  **zero** skipped rules, and `tests/smoke.rs` asserts it.
- One local patch to upstream data, pending a submitted fix: `bg.sor`
  had a bracket and paren transposed in an ordinal-thousands rule
  (`\2)])` instead of `\2))]`) — a plain typo rather than a deliberate
  construct, confirmed by comparing against the correctly nested
  singular rule right above it. See
  [data/SOURCE.md](data/SOURCE.md#local-patches-pending-upstream) and
  AGENTS.md for the full reasoning, including the engine-side fixes
  considered and rejected. `scripts/sync-data.sh` reapplies it until
  [libnumbertext#139](https://github.com/Numbertext/libnumbertext/issues/139)
  is merged.
- A CLI (`src/bin/numbertext.rs`), installable with
  `cargo install numbertext` and doubling as a runnable usage example.
  Its argument parsing is hand-rolled so that no parsing crate lands in
  the dependency tree of library consumers, and its flags are part of the
  public API (see "API stability" in the README).
- Conditional text (`[...]`) is decided by the **first** call inside the
  block, matching the reference implementations. They rewrite
  `[before$CALL after]` into `$(SEP before SEP $CALL SEP after)` and let
  two hidden rules keep or drop it, so only one call can ever decide.
  Requiring every call to be non-empty looked equivalent and was not:
  `lg.sor`'s `1(\d{6}) kakadde [ka$(:1) ne $1]` has an empty second call
  for a round million, and the block was being discarded.
- Shell completions for bash, zsh and fish
  ([`completions/`](completions)), hand-written rather than generated so
  that no CLI-parsing crate enters library consumers' dependency tree.
  `--lang` resolves its candidates from `numbertext --list` at completion
  time, so it tracks the languages a given build embedded;
  `tests/completions.rs` checks the flag list against the CLI's own
  `--help` so the two cannot drift.
- `README.md`/`README.pt-BR.md`, `CONTRIBUTING.md`/`.pt-BR.md`,
  `docs/coding-style.md`/`.pt-BR.md`, `AGENTS.md` (which `CLAUDE.md`
  imports, so Codex/Cursor/Claude Code all read one file), and this
  changelog. Both READMEs' examples are compiled and run as doctests.
- Quality tooling wired into `make` and CI: `clippy` (`pedantic` +
  `cargo`, `-D warnings`), `rustfmt`, `cargo-deny` (advisories,
  licenses, dependency bans), `cargo-mutants` for mutation testing, and
  `markdownlint-cli2`. CI tests against the declared MSRV (1.85) and
  `stable`, and runs lint exactly once outside that matrix.
- AI-assistant and editor scaffolding: `.claude/settings.json` (a scoped
  permission allowlist plus a `Stop` hook surfacing `doc-sync`),
  `.claude/agents/` (`api-surface-reviewer` and `bilingual-doc-reviewer`,
  read-only reviewers), `.vscode/`, and
  [skills/using-numbertext/SKILL.md](skills/using-numbertext/SKILL.md),
  a Claude Skill documenting the public API for projects that depend on
  this one.

[0.1.0]: https://github.com/casara/numbertext-rs/releases/tag/v0.1.0
