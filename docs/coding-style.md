# Rust Style Guide

*[Leia em português](coding-style.pt-BR.md)*

## Purpose

This document records the conventions this project adopts **beyond** what
`rustfmt` and `clippy` already enforce automatically. If a tool can check
it, it isn't written down here — it's in `rustfmt.toml`, `clippy.toml`,
or the workspace lint table in `Cargo.toml`.

Decisions prioritize, in order:

* readability;
* consistency;
* maintainability;
* diff quality.

---

## Formatting

All code is formatted with `rustfmt`, at its defaults. `rustfmt.toml`
sets only the edition.

That's deliberate: rustfmt's defaults are what every Rust reader already
expects, and a house style costs more in unfamiliarity than it buys in
preference. Don't hand-format against the tool.

---

## Lints

The workspace lint table in `Cargo.toml` turns on `clippy::pedantic` and
`clippy::cargo` for every crate, and CI runs clippy with `-D warnings`.

When a pedantic lint is wrong for a specific case, prefer fixing the code
over silencing the lint. `render_node` taking four loose booleans was a
real design smell that `clippy::fn_params_excessive_bools` caught
correctly; the fix was the `Boundary` type, not an `#[allow]`.

Where an `#[allow]` genuinely is right, scope it as narrowly as possible
and say why in a comment on the same line or just above it. A bare
`#[allow]` with no reason is a TODO nobody will ever come back to.

---

## Readability over conciseness

Prefer explicit, easy-to-read code over maximally compact code.

Preferred:

```rust
let Some(caps) = rule.re.captures(input) else {
    continue;
};
```

Avoid chains that pack three decisions into one expression when they
would read as three steps.

---

## Comments

Comments should explain:

* purpose;
* behavior;
* limitations;
* **design decisions, especially the alternatives that were rejected.**

That last one carries most of the weight in this repository. The
expensive knowledge here isn't what the code does — it's why it isn't
done the other, apparently simpler way. A reader who doesn't find that
written down will eventually "simplify" it back.

Comments that restate the item's name should be avoided.

Bad:

```rust
/// Compiles a program.
pub fn compile(source: &str, lang: &str) -> Self
```

Better:

```rust
/// Compiles Soros source code into a runnable program for the given
/// language code.
///
/// This is infallible by design. There is no such thing as a source that
/// fails to tokenize - every failure is per-rule and is recorded in
/// [`Program::skipped`] instead, so one malformed line never takes a
/// whole locale down with it.
```

---

## Doc comments are tests

Every public item's example is a doctest, run by `cargo test --doc`. Both
READMEs are wired in as doctests too (see `ReadmeExamples` in
`src/lib.rs`), so their examples are compiled and run rather than merely
plausible.

Write examples that assert, not examples that print:

```rust
/// ```
/// # let en = numbertext::Locale::new("en").unwrap();
/// assert_eq!(en.ordinal("123"), "one hundred twenty-third");
/// ```
```

An example with `assert_eq!` is a specification the compiler checks. An
example with `println!` only proves the code compiles.

**Run the code to get the expected value.** Locale output encodes real
per-language stylistic choices that are easy to get wrong by intuition —
see AGENTS.md's "Verify locale output before writing it down."

---

## Errors

The public error type is a `#[non_exhaustive]` enum with one variant per
distinguishable failure, matched by callers rather than parsed from its
message.

### Model what can actually fail

The most important error-handling decision in this crate is what *isn't*
in `Error`. Conversion cannot fail: the Soros language has no notion of a
failed conversion, so input no rule matches yields an empty string.
`Locale`'s conversion methods return `String`, not `Result<String, _>`.

Resist the pull to make everything fallible for symmetry. A `Result` the
caller can only `.unwrap()` is noise that trains people to ignore the
ones that matter.

### Messages

Lowercase, no trailing period, describing what could not be done — and
naming the thing that could not be done to.

Correct:

```rust
write!(f, "unknown language {lang:?} (see `languages()` for the supported base codes)")
```

Avoid generic wrappers such as "error", "failed", or "unexpected error",
and avoid repeating context a caller already has.

### Message text is not API

Callers match on variants. That's why the wording can carry a full
sentence of guidance without being a compatibility hazard.

---

## Panics

Library code does not panic on caller input. `expect` is acceptable only
where the invariant is guaranteed by this code rather than by a caller —
a regex literal written in this file, an environment variable Cargo
always sets in a build script — and the `expect` message says which
invariant is being relied on:

```rust
Regex::new(r"\[:([^:\]]+):\]").expect("built-in lang-tag pattern must compile")
```

---

## Dependencies

Adding one is a decision, not a formality. `numbertext` has exactly one
direct dependency (`soros`), which itself has exactly one
(`fancy-regex`). That's why the CLI's argument parsing is hand-rolled: a
binary target's dependencies are the package's dependencies, so a parsing
crate would land in the tree of everyone who depends on the library and
never runs the binary.

Before adding a dependency, ask what it would cost the consumer, and
whether the code it replaces is genuinely subtle enough to be worth it.

---

## Visibility

Public means supported. Anything not deliberately part of the API is
private, and `pub(crate)` where crossing a module boundary inside a crate
is genuinely needed.

The `soros`/`numbertext` split is the same principle at crate scale: the
interpreter is a separate crate so the boundary is enforced by Cargo
rather than by convention. Keep `soros` free of any numbertext concept —
if a change would have it know what an "ordinal" is, the change belongs
on the other side of the boundary.
