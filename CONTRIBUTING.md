# Contributing

*[Leia em português](CONTRIBUTING.pt-BR.md)*

This project follows the [Code of Conduct](CODE_OF_CONDUCT.md). By
participating, you're expected to follow it.

## Before you start

Read, in this order:

1. [AGENTS.md](AGENTS.md) — architecture, the Soros language condensed,
   known/deliberate limitations, non-obvious design decisions.
2. [docs/coding-style.md](docs/coding-style.md) — code conventions
   beyond what `rustfmt` and `clippy` already enforce.
3. [README.md](README.md) — the user-facing API and usage examples.
4. [data/SOURCE.md](data/SOURCE.md) — how the bundled `.sor` locale
   files are sourced and how to add a new one.

## Environment

`make help` lists every command. Before opening a PR:

```sh
make check     # fmt-check + lint + msrv + spell + tests (incl. doctests) + feature selection
make lint-md   # markdown lint (requires Node.js >= 20)
```

That's the minimum CI runs on every PR
([.github/workflows/ci.yml](.github/workflows/ci.yml)) — split across
separate jobs there (`test` per Rust version, `lint` once, plus markdown
lint, `cargo-deny`, and the data provenance check) rather than one
`make check` call. See AGENTS.md's "Commands" section for why that split
is deliberate and shouldn't be merged.

If the change touches behavior — not just refactoring — include a test
covering the new case. For a new or changed language, add a handful of
known-correct conversions **verified by actually running the code**
(`make run ARGS="--lang <x> --cardinal <n>"`), not guessed. See AGENTS.md's
"Verify locale output before writing it down."

## Branch workflow

Rebase, not merge commits: update your branch with `git rebase` against
the base before opening or updating a PR, instead of merging the base
into your branch. Linear history, no merge commits.

The commit convention below is mandatory on `main`. A work branch that
will be squashed on merge doesn't need to follow it strictly — the
intermediate history isn't what stays.

## Commit messages

[Conventional Commits](https://www.conventionalcommits.org/), in
English, imperative mood:

```text
<type>(<scope>): <description>
```

* `<description>` in the imperative, describing the action (`add`,
  `fix`, `remove`, `harden`, `resolve`, `guarantee`), not the resulting
  state (`added`) or history (`fixed`, "adds/added").
* `<scope>` is the affected crate or area. It's not a closed list, but
  the most common ones are:

| Scope | Covers |
| --- | --- |
| `soros` | the interpreter crate (`crates/soros`) |
| `numbertext` | the public API, `Locale`, the registry |
| `cli` | the `numbertext` binary (`src/bin`) |
| `data` | the bundled `.sor` locale files and their provenance |
| `build` | `build.rs`, the generated locale table, Cargo features |
| `docs` | README/AGENTS/CONTRIBUTING/skill prose |
| `ci` | `.github/workflows/` |
| `deps` | dependency bumps |
| `release` | version bumps, changelog cuts |

### Types

| Type | When to use |
| --- | --- |
| `feat` | New feature (semver MINOR). |
| `fix` | Bug fix (semver PATCH). |
| `docs` | Documentation only (README, doc comments) — no code change. |
| `test` | Test only (added, changed, or removed) — no production code change. |
| `refactor` | Changes how code is written/organized without changing observable behavior. |
| `perf` | A change whose purpose is performance. |
| `style` | Formatting, whitespace, lint — no code change. |
| `build` | Build and dependencies (`Cargo.toml`, `build.rs`, `Makefile`, tooling). |
| `ci` | Continuous integration (`.github/workflows/`). |
| `chore` | Maintenance that doesn't fit the types above (config, `.gitignore`, ...). |
| `cleanup` | Removes commented-out, dead, or unnecessary code — no behavior change. |
| `remove` | Removes an obsolete/unused file, directory, or feature. |
| `raw` | A configuration/data file change that doesn't fit the above (e.g. a `data/*.sor` sync). |

`make commit-check` lints your branch against this convention
(`RANGE=origin/main..HEAD` to limit it to your own commits), and CI runs
the same check on every pull request. The three types below that are not
part of Conventional Commits — `cleanup`, `remove`, `raw` — are declared
in `cog.toml`, so they pass.

### Examples

```text
feat(numbertext): add Locale for resolving a language once
fix(soros): propagate the leading boundary through a piped call argument
docs(readme): explain regional variant fallback for en-GB/pt-BR
raw(data): sync data/*.sor to libnumbertext@a4b0225
```

## Changing the public API

The project is on **v0.x**, so a breaking change is allowed — see "API
stability" in the README for what that means for users. It still has to
be deliberate and visible:

* Say so in the PR description, and add a `### Changed` or `### Removed`
  entry to [CHANGELOG.md](CHANGELOG.md) with the migration in the same
  entry.
* Where a rename can keep the old spelling compiling, keep it as a
  `#[deprecated]` alias rather than deleting it outright.
* Prefer an additive change when one exists: a new method alongside the
  old one, a new optional parameter via a builder.

Raising the MSRV is a breaking change under this project's rules, and
gets the same treatment.

The `numbertext` binary's flags count as public API too — see the
README. Renaming or removing one is a breaking change and goes through
everything above, the same as a function signature would.

Error message text is explicitly *not* part of the public API; matching
on it is not supported. `Error`'s variants are. Neither is the wording of
`--help`.

## Working with the bundled data

`data/*.sor` comes from upstream and is not hand-edited — see AGENTS.md.
`make check-data` verifies that claim against upstream's own blob hashes,
and CI runs it on every PR, so a stray edit fails visibly rather than
quietly becoming this project's private fork of a rule file.

Adding or removing a locale file also means `make gen-features`: Cargo
can't discover features at build time, so the per-language feature list
in `Cargo.toml` is generated from the data directory.

## Contributing with an AI assistant

This project is built with one, and there's nothing to hide or disclose
about that: the code is judged the same either way, and a PR isn't
marked.

What's asked is the same thing asked of anyone:

* **Understand what you're submitting.** If you can't explain why a
  change is correct, it isn't ready — regardless of what wrote it.
* **Verify locale output, don't guess it.** A `.sor` file encodes real,
  sometimes surprising per-language stylistic choices (see AGENTS.md's
  own examples: `en` vs `en-GB`'s "and"; `cardinal` vs `year`'s "and"
  removal). Run the code before writing an expected value down.
* **Don't let a tool re-litigate a settled decision.** AGENTS.md
  documents this port's design decisions — the single-regex-crate engine,
  the rejected probe-and-verify alternative, the `bg.sor` data patch —
  because each boundary was reached once, on purpose. A PR "fixing" one
  by guessing at the reasoning gets a pointer, not a debate.
* **Run the checks locally.** `make check` and `make lint-md`, before
  opening the PR rather than after CI says so.

AGENTS.md is read by Codex and Cursor directly; CLAUDE.md imports it for
Claude Code. Anything you add for one of them belongs in AGENTS.md, so
the others get it too.

## Pull requests

* `make check` passing is mandatory, not optional. If the change touches
  any Markdown file, `make lint-md` too.
* A small PR focused on one change is preferable to a large one covering
  several unrelated things — but that's judgment, not a strict rule.
* If the change alters documented behavior, update the docs in the same
  PR. Both languages: an edit to `README.md` that skips
  `README.pt-BR.md` leaves the pair contradicting itself.
