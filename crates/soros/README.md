# soros

[![crates.io](https://img.shields.io/crates/v/soros.svg)](https://crates.io/crates/soros)
[![docs.rs](https://img.shields.io/docsrs/soros)](https://docs.rs/soros)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

An interpreter for the **Soros** programming language: the small
regex-rewriting DSL the [Numbertext.org](https://numbertext.github.io/)
project uses to describe number-to-text conversion, one rule file per
language.

Leia isto em [português (Brasil)](README.pt-BR.md).

```rust
// The specification's own string-reversal program.
let program = soros::Program::compile(r"(.*)(.) \2$1", "");

assert_eq!(program.run("hello"), "olleh");
```

## Scope

This crate implements the language and nothing else. It has no notion of
numbers, cardinals, currencies, or languages — everything
numbertext-specific lives in the
[`numbertext`](https://crates.io/crates/numbertext) crate, which is the
usual reason to reach for this one.

Keeping the two apart is the point, and it is why this is a separate
crate rather than a private module: `soros` has no dependency on
`numbertext`, so the boundary is enforced by Cargo rather than by a
convention someone has to remember.

## The language, condensed

The full specification is `doc/sorosspec.pdf` in the
[upstream repository](https://github.com/Numbertext/libnumbertext).

- A program is one flat, ordered list of `pattern → template` rules,
  always applied as a **full match** against the current input string.
  First match wins; no match yields an empty string.
- `== prefix ==` defines a macro: the prefix is concatenated onto every
  following rule's pattern until the next `==`.
- Template syntax: `\N` (raw backreference), `$N` (sugar for `$(\N)`),
  `$(...)` (recursive call on a constructed argument), `[...]`
  (conditional text, kept only if the calls inside it are non-empty),
  `|` (boundary modifier).
- A hidden rule is always active, stripping extra leading zeros.

## Regex engine

Patterns compile through [`fancy-regex`](https://docs.rs/fancy-regex),
which delegates to the linear-time [`regex`](https://docs.rs/regex)
engine for any pattern without a backreference or lookaround, and uses
its own backtracking engine only for the rest — which real rule files
genuinely need. Backtracking is bounded by `BACKTRACK_LIMIT`, a step
count rather than a wall-clock timeout, so behavior is reproducible
across machines. Evaluation is bounded by `MAX_RECURSION_DEPTH`.

## Robustness

`Program::compile` is infallible. A rule it cannot compile is recorded in
`Program::skipped` rather than failing the whole program, so one
malformed line never takes a rule file down with it — and is reported,
never guessed at.

## License

MIT, see [LICENSE](LICENSE). Note that the `.sor` **rule files** shipped
by the `numbertext` crate are a different matter: they come from upstream
libnumbertext under BSD-3-Clause. This crate ships no rule files.
