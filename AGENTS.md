# AGENTS.md

Guidance for AI coding agents working in this repository. `CLAUDE.md`
imports this file (`@AGENTS.md`) rather than duplicating it, so Codex,
Cursor and Claude Code all read the same instructions from one place.
See [README.md](README.md) / [README.pt-BR.md](README.pt-BR.md) for the
user-facing overview.

## What this is

`numbertext-rs` spells out numbers as words - cardinals, ordinals,
gender/case variants, negatives, currency amounts - in every language
shipped by [Numbertext.org](https://numbertext.github.io/) (upstream:
[libnumbertext](https://github.com/Numbertext/libnumbertext)). Rather
than hand-writing per-language conversion code, it interprets the same
`.sor` rule files upstream ships, written in a small regex-rewriting DSL
called **Soros**. Adding a language is a data change
(`data/<code>.sor`), never a code change.

## Architecture

```text
crates/soros/          the Soros language interpreter - a separate crate
  escape.rs             hides/reveals the spec's backslash-escaped special characters
  lexer.rs              comment/language-line filtering, statement splitting, pattern/value split
  template.rs           parses a rule's replacement text into an AST (literal/backref/call/optional/pipe)
  regex.rs              pattern compilation and the backtracking bound
  program.rs            Program::compile(): ordered rule list, macro-prefix concatenation, hidden left-zero rule
  eval.rs               Program::run(): the recursive evaluator (first full-match rule wins, boundary propagation)

src/lib.rs             public API: Locale + the one-shot free functions, register_locale, languages
src/error.rs           the Error enum (UnknownLanguage, EmptyLocaleCode)
src/registry.rs        process-wide locale registry: lazy compile-and-cache, language-code resolution
src/bin/numbertext.rs  CLI wrapper, also a runnable usage example
build.rs               generates the embedded-locale table from the selected `lang-*` features
data/*.sor             upstream rule files, byte for byte - see data/SOURCE.md
```

### Why `soros` is a separate crate, not a private module

The interpreter knows nothing about numbers, currencies, or languages.
Everything numbertext-specific - which section name means "ordinal", how
a currency amount string is assembled - lives in the root crate.

That separation is enforced by Cargo: `soros` does not depend on
`numbertext`, so there is simply nothing to reach back through. A
private `mod soros` inside one crate would have left the same boundary
resting on convention, needing an architecture linter to hold it in
place. Here the compiler holds it, and the interpreter is independently
usable and independently documented as a result.

The cost is real and worth stating: two crates means two versions to
keep in step on release. They share one `[workspace.package]` version
for exactly that reason.

## The Soros language, condensed

Full spec: `doc/sorosspec.pdf` in the upstream repo (read while building
this port; not vendored here). The essentials:

- A program is one flat, ordered list of `pattern → template` rules,
  always matched as a **full match** against the current input string.
  First match wins; no match yields an empty string.
- `== prefix ==` defines a **macro**: the prefix text is concatenated (as
  literal regex text, with `^` hoisted to the front when present) onto
  every following rule's pattern, until the next `==`. There is no
  separate namespace per prefix - `$(ordinal 123)` just re-runs the whole
  rule list against the literal string `"ordinal 123"`, and whichever
  macro-prefixed rule matches that literal text wins.
- Template syntax: `\N` (raw backreference, N=0-9, no re-evaluation),
  `$N` (sugar for `$(\N)`), `$(...)` (recursive call to the whole engine
  on a constructed argument), `[...]` (conditional text - kept only if
  the call(s) inside are non-empty), `|` (boundary-modifier marker
  adjacent to a call, forcing leading/trailing-boundary eligibility
  regardless of position, e.g. for `negative |$1` after stripping a
  sign).
- A hidden rule is always active: `([a-z][-a-z]* )?0+(0|[1-9]\d*)` →
  `$(\1\2)`, stripping extra leading zeros, also after a macro-prefix
  word.

## Regex engine: one crate, not two glued together

Rules compile through [`fancy-regex`](https://docs.rs/fancy-regex),
which parses a pattern and delegates it wholesale to the linear-time
[`regex`](https://docs.rs/regex) crate whenever it contains no
backreference or lookaround - which is every rule but a handful across
all of `data/*.sor`. Only the rest run on its backtracking engine.

The fallback is not optional. The `regex` crate, like Go's RE2-based
`regexp`, deliberately excludes backreferences, and at least one real
locale file needs one *inside a matching pattern*: `hu.sor` encodes a
grammatical-suffix "database" as a single packed string and looks
entries up with `.*:\1:(-[^-:]*).*`, where `\1` correlates the requested
case name with its entry. The reference C++/Java/Python implementations
get this free from their native backtracking engines.

Every backtracking match is bounded by `soros::BACKTRACK_LIMIT`, so a
pathological pattern degrades to "this rule doesn't match" instead of
hanging - the standard ReDoS mitigation.

Two properties of this design are worth not re-deriving:

- **The bound is a step count, not a wall-clock timeout.** That makes it
  deterministic: the same pattern and input either fit the budget or do
  not, identically on every machine. A Go port has to use a time-based
  bound instead (`regexp2` only offers `MatchTimeout`), which makes the
  same call succeed on a fast machine and fail on a loaded CI runner.
- **Delegation is decided from the parsed pattern, not from a failed
  compile.** A Go port has to try RE2 first and fall back when
  `regexp.Compile` returns an error, which means two engines, two
  dialects, and a rule that can silently change meaning depending on
  which side it landed on. Here there is one dialect.

`tests/regex.rs` in the soros crate pins all of this: that a
backreference matches, that lookaround works, that a backreference-free
catastrophic pattern stays linear, and that a genuinely catastrophic one
degrades instead of hanging.

### A cheaper design considered and rejected

Matching a backreferenced pattern by compiling a "probe" under the
linear engine - the backreference position replaced by a plain copy of
the referenced group's own pattern - then verifying group equality after
the fact. That avoids the backtracking dependency entirely and works for
every backreference in `data/*.sor` today.

It is also provably incomplete. A linear engine picks one match
deterministically, so for a pattern like `(a+)\1` against a string with
more than one valid split, that single candidate can fail the equality
check even though a valid split exists elsewhere - a false negative,
which shows up as a rule silently not firing rather than as an error.
`backreference_finds_a_non_greedy_split` in `crates/soros/tests/regex.rs`
is that exact case, kept as a regression test.

## The `bg.sor` rule that is an upstream typo, not a parser gap

One `bg.sor` rule's `[...]`/`$(...)` spans cross rather than nest:
`[ $(and:$(ordinal \2)])` - the bracket's `]` sits inside the call's
still-open parens, so the call's own paren matching (which correctly
ignores `]`) consumes it before the bracket's scan ever sees it as a
closer.

This is a data bug, and it is fixed in the data. Comparing the broken
line against its immediate neighbor - `1([0-9]{3}) хиляда[
$(and:$(ordinal \1))]`, the singular counterpart, correctly nested -
shows the crossing was never intentional: upstream's line ends `\2)])`
where the sibling rule (and the general `$(eN:\1)[ $(and:$2)]` shape
every other magnitude uses) has `))]`. A two-character transposition,
verifiable by inspection against a rule that already works, not a guess
about intent.

Loosening the parser to tolerate crossing spans was considered and
rejected: there is no principled tree for a genuine crossing, so
building one means either picking an arbitrary resolution or bit-for-bit
replicating the reference implementations' own. And theirs is not a
semantic at all - it is a text substitution. Both the C++
([`src/Soros.cxx:125`](https://github.com/Numbertext/libnumbertext/blob/master/src/Soros.cxx#L125),
`translate(s2, L"]", L")", L"")`) and the Python
([`src/Soros.py:75`](https://github.com/Numbertext/libnumbertext/blob/master/src/Soros.py#L75),
`re.sub(r"]", ")", s2)`) rewrite **every** `]` to `)` unconditionally,
before any paren matching runs, so a bracket never participates in
nesting analysis and the malformed line comes out balanced by accident.
An implementation accident is not worth encoding permanently into an
interpreter.

So the fix lives in `data/bg.sor` (`\2)])` → `\2))]`), documented in
`data/SOURCE.md`'s "Local patches" section and reapplied on every sync
by `scripts/sync-data.sh`. Reported upstream as
[libnumbertext#139](https://github.com/Numbertext/libnumbertext/issues/139),
still open - `scripts/check-data-provenance.py` fails if the patch
stops being needed, which is how the eventual merge gets noticed.

The general principle stands for whatever the *next* unparsable rule
turns out to be: a rule `Program::compile` cannot parse is recorded in
`Program::skipped` and never guessed at. What changed here is
recognizing that this particular "can't parse" was upstream's bug rather
than this port's limitation - worth checking before assuming the fix
belongs in Rust code at all.

**Every bundled locale currently compiles with zero skipped rules**, and
`tests/smoke.rs` asserts exactly that. A skipped rule simply never
matches, so a locale carrying one keeps working for most input and
quietly produces wrong wording for the rest; the test is what turns that
silent degradation into a failure.

## Language-code resolution follows upstream, including its exceptions

`registry::resolve` tries, in order: the code exactly as given, the code
with `-` rewritten to `_`, the base before the first `-`, and finally the
Norwegian aliases. That mirrors `Numbertext::load` in upstream's
`src/Numbertext.cxx`, and each step earns its place:

- **Exact first** is this port's own addition. Upstream looks up files;
  this registry also holds codes added by `register_locale`, which may
  contain a `-` that must not be rewritten out from under the caller.
- **`-` to `_`** is what lets `hu-Hung` reach `hu_Hung.sor`. Skipping it
  does not produce an error - it falls through to `hu.sor` and returns
  ordinary Hungarian where Old Hungarian script was asked for. A silently
  wrong answer, which is why it has a test of its own.
- **Norwegian** (`nb`, `nn`, `nb-NO`, `nn-NO` → `no.sor`) is upstream's
  one hard-coded exception, `LANG_PATTERN_NO`. Without it those codes are
  rejected outright *and* `no.sor`'s `[:nn:]`/`[:nn-NO:]` lines become
  unreachable by any code at all - dead data.

Whatever source is found is always compiled with the **full requested
code**, which is what activates the tagged lines (spec 2.7.1).

Note what resolution deliberately does *not* do: reject an unknown
suffix. `en-ZZ` resolves to `en.sor` and returns plain English, the same
as upstream. So the set of accepted codes is unbounded, and
`languages()`/`language_variants()` describe the codes that *change the
output*, not the codes that work.

## `[...]` is decided by its first call, not by all of them

Spec 2.4.5 reads as though a conditional block is kept when "the call(s)
inside" are non-empty, and "every call must be non-empty" is the obvious
implementation. It is wrong, and only one line in all of `data/*.sor`
proves it.

The reference implementations never evaluate a bracket. They rewrite
`[before$CALL after]` into `$(SEP before SEP $CALL SEP after)` with a
regex substitution and inject two hidden rules -
`SEP(.*)SEP(.+)SEP(.*)` keeps the block, `SEP.*SEPSEP.*` drops it. The
substitution moves exactly one call into that middle slot, the first, so
only the first can decide; anything after it is ordinary content that may
render empty without consequence.

`lg.sor` line 43 is the case that tells them apart:
`1(\d{6}) kakadde [ka$(:1) ne $1]`. For a round million the second call
is empty, so "every call" discards the block and `1000000` comes out
`kakadde` instead of `kakadde kamu ne` (with a trailing space, which
upstream emits too). Nothing else in the bundled data
distinguishes the two rules, which is why a full cross-check against
upstream found it and 114 tests did not.

`conditional_text_is_decided_by_its_first_call` in
`crates/soros/tests/spec.rs` pins the semantics on a minimal program;
`luganda_keeps_a_block_whose_later_call_is_empty` in
`tests/numbertext.rs` pins the real locale. Both expected values come
from running upstream's own `spellout`, trailing space included.

## Working in this repo

- **Verify locale output before writing it down.** The rule files encode
  real per-language stylistic choices that are easy to get wrong by
  intuition alone. In `en.sor` alone: plain `en` omits "and" in "one
  hundred one" while `en-AU`/`en-GB`/`en-IE`/`en-NZ` include it;
  `cardinal("1001")` is "one thousand and one" (the separator function
  inserts "and" before a sub-100 remainder); but `year("2001")` is "two
  thousand one", because the `year` section runs its cardinal result
  through a `year-remove-and` pass that plain `cardinal` never sees. When
  adding a test expectation, run the code
  (`make run ARGS="--lang <x> --cardinal <n>"`) and confirm the result,
  rather than guessing.
- **Engine changes**: validate against `crates/soros/tests/spec.rs`
  first (the spec's own canonical examples - string reversal, thousands
  separator, Devanagari digits - plus targeted boundary/bracket/macro
  cases) and `crates/soros/tests/regex.rs` (the delegation and
  backtracking-bound boundary) before touching real locale files. If the
  engine tests pass but a real `.sor` file misbehaves, that means the
  spec has a subtlety the unit tests do not cover yet - add a minimal
  reproduction there rather than debugging against a 300-line locale.
- **Don't hand-edit `data/*.sor`.** These are synced verbatim from
  upstream (see `data/SOURCE.md`); fix bugs in the interpreter, not the
  data - unless the data is actually wrong, as in the one documented,
  narrowly-justified exception above. Use `scripts/sync-data.sh` to
  refresh; it reapplies that patch automatically, and
  `make check-data` verifies the result against upstream's blob hashes.
- **Adding or removing a locale means `make gen-features`.** The
  per-language Cargo features are generated from the `data/*.sor` file
  list by `scripts/gen-locale-features.py`, between marker comments in
  `Cargo.toml`. Cargo cannot discover features at build time, so this is
  the one place a data change touches a non-data file.
- Code comments and identifiers are English-only, even though the
  project's prose ships in English and pt-BR. The bilingual pairs are
  `README.md`, `CONTRIBUTING.md`, `docs/coding-style.md` and
  `crates/soros/README.md`; editing one half without the other leaves the
  pair contradicting itself, which `bilingual-doc-reviewer` exists to
  catch. Every README example on both sides is a doctest, so a translated
  example that drifts fails `cargo test --doc` rather than quietly
  misleading a reader. (crates.io shows only the English half - the
  Portuguese one ships and is read on GitHub.)
- Conventions beyond what `rustfmt`/`clippy` already enforce are in
  [docs/coding-style.md](docs/coding-style.md) /
  [docs/coding-style.pt-BR.md](docs/coding-style.pt-BR.md).

## Commands

`make help` lists every command with a one-line description. Before
opening a PR:

```sh
make check     # fmt-check + lint + msrv + spell + tests (incl. doctests) + feature selection
make lint-md   # markdown lint (requires Node.js >= 20)
```

That is the minimum CI runs on every PR
([.github/workflows/ci.yml](.github/workflows/ci.yml)), split there
across separate jobs rather than one `make check` call.

**The split is load-bearing, not cosmetic.** `test` runs per toolchain
in a matrix (the declared MSRV plus `stable`); `lint` runs **once**, on
`stable`, deliberately outside that matrix. clippy and rustfmt are dev
tools whose own minimum supported version moves independently of this
crate's declared floor, and new clippy releases add lints older
toolchains do not have - so linting inside the MSRV matrix means an
unrelated tool release breaks the MSRV leg. Linting has no
per-toolchain behavior worth matrixing anyway. Do not "simplify" CI by
merging them.

### `rust-toolchain.toml` deliberately has no `channel`

rustup accepts a `[toolchain]` section that lists only `components`, and
this repository uses exactly that. Adding `channel = "stable"` would look
harmless and would quietly break the MSRV guarantee: `rust-toolchain.toml`
outranks the `rustup default` that `dtolnay/rust-toolchain` performs, so
the `1.85` leg of the CI matrix would run on stable, the floor would go
unverified, and CI would stay green while doing it.

Verified rather than assumed - the action's `action.yml` uses `rustup
default` and never sets `RUSTUP_TOOLCHAIN`, and a local test confirmed a
channel in this file wins over it. The `verify the active toolchain` step
in the `test` job exists to make that failure loud if anyone adds a
channel later.

The version lives in three places that do not contradict each other:
`rust-version` in `Cargo.toml` is the promised floor, `.tool-versions` is
what asdf installs locally, and the CI matrix picks per job.

`make test-mutation` runs mutation testing. Both its flags are
load-bearing too: `--workspace` because cargo-mutants otherwise only
mutates the package in the current directory, and `--test-workspace`
because it otherwise runs **only the mutated package's own tests** - a
mutation in `soros` that only the `numbertext` crate's tests would catch
gets reported as a surviving mutant nothing covers. Verified against
cargo-mutants' documented behavior; a verdict produced without both
flags is not worth reading.

`make test-nextest` runs the suite through cargo-nextest, which is
faster and stricter - each test gets its own process, so a test that only
passed because an earlier one in the same process had already populated
the shared locale registry fails there and not under `cargo test`. It
**cannot run doctests**, which is why that target runs `cargo test --doc`
as a second step; dropping that line would silently stop checking 39
documentation examples, including both READMEs.

`make test-feature-matrix` builds each `lang-*` feature on its own. There
are 50+ of them and `make check` only exercises three configurations, so
this is what would catch a locale whose file name does not round-trip
through `scripts/gen-locale-features.py`'s name mangling - a feature
nothing can ever enable. It is dozens of builds, so it stays out of
`check`.

Four smaller checks round out the set. `make spell` runs `typos`, scoped
by `_typos.toml` - which excludes `data/` (52 languages of legitimate
words a checker for English typos can only misjudge) and every
`.pt-BR.md` (typos ships no Portuguese dictionary, so on those it
reports only false positives - ordinary Portuguese words read as
misspelled English). **The Portuguese prose is
therefore not spell-checked**; `bilingual-doc-reviewer` and the doctests
in both READMEs are what cover it.

`make unused-deps` (cargo-machete) and `make commit-check` (cocogitto,
reading `cog.toml`) are the other two. `cog.toml` has to declare
`cleanup`, `remove` and `raw` explicitly, because CONTRIBUTING.md adds
them to the conventional set and cocogitto would otherwise reject this
project's own documented convention.

`make msrv-find` is deliberately separate from `make msrv`. The latter
checks the declaration is self-consistent, and CI's matrix checks it
works; only `msrv-find` can say whether the floor is *higher than it
needs to be*, by bisecting toolchains. It is advisory - lowering the
floor widens what the project promises, so it stays a decision rather
than an automatic bump.

It has been run, and the answer is already known: **the floor is set by
edition 2024, not by anything this code does.** 1.84 fails with "the
package requires the Cargo feature called `edition2024`", and 1.85 passes
the whole suite. So `rust-version = "1.85"` is exact rather than
cautious, and the only lever that would lower it is moving back to
edition 2021 - a much bigger trade than an MSRV bump. Re-run `msrv-find`
after an edition change or a new dependency, not routinely.

`make doc-sync` prints which documentation a diff touching the public
API has not touched. It never fails: a change can legitimately leave all
of them alone, and a check that cried wolf would be ignored. A `Stop`
hook in `.claude/settings.json` surfaces the same output at the end of a
turn.

For a closer look, the `api-surface-reviewer` subagent
(`.claude/agents/`) reads the diff in a clean context and reports what
changed in the public API with `file:line`, and which documentation
contradicts it. `bilingual-doc-reviewer` (same directory) does the
equivalent check between an English doc and its `.pt-BR.md` twin.

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full workflow (commit
convention, branch/PR process, doc-sync expectations).
