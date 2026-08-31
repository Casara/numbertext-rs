//! An interpreter for the Soros programming language.
//!
//! Soros is the small regex-rewriting DSL the
//! [Numbertext.org](https://numbertext.github.io/) project uses to
//! describe number-to-text conversion on a per-language basis. This
//! crate implements the language and nothing else: it has no notion of
//! numbers, cardinals, currencies, or languages. Everything
//! numbertext-specific - which section name means "ordinal", how a
//! currency amount is assembled - lives in the
//! [`numbertext`](https://docs.rs/numbertext) crate, which is the usual
//! reason to reach for this one.
//!
//! Keeping the two apart is the point: the boundary is enforced by
//! Cargo, since this crate simply has no dependency to reach back
//! through, rather than by a convention someone has to remember.
//!
//! # The language, condensed
//!
//! The full specification is `doc/sorosspec.pdf` in the
//! [upstream repository](https://github.com/Numbertext/libnumbertext).
//! The essentials:
//!
//! - A program is one flat, ordered list of `pattern → template` rules,
//!   always applied as a **full match** against the current input
//!   string. First match wins; no match yields an empty string.
//! - `== prefix ==` defines a **macro**: the prefix text is
//!   concatenated (as literal regex text, with `^` hoisted to the front
//!   when present) onto every following rule's pattern, until the next
//!   `==`. There is no separate namespace per prefix - `$(ordinal 123)`
//!   just re-runs the whole rule list against the literal string
//!   `"ordinal 123"`, and whichever macro-prefixed rule matches wins.
//! - Template syntax: `\N` (raw backreference, N=0-9, not re-evaluated),
//!   `$N` (sugar for `$(\N)`), `$(...)` (recursive call to the whole
//!   engine on a constructed argument), `[...]` (conditional text, kept
//!   only if the call(s) inside it are non-empty), and `|` (a
//!   boundary-modifier marker adjacent to a call, forcing
//!   leading/trailing-boundary eligibility regardless of position - what
//!   `negative |$1` needs after stripping a sign).
//! - A hidden rule is always active: `([a-z][-a-z]* )?0+(0|[1-9]\d*)` →
//!   `$(\1\2)`, stripping extra leading zeros, also after a
//!   macro-prefix word.
//!
//! # Example
//!
//! The specification's own string-reversal program:
//!
//! ```
//! let program = soros::Program::compile(r"(.*)(.) \2$1", "");
//!
//! assert_eq!(program.run("hello"), "olleh");
//! ```
//!
//! # The regex engine
//!
//! Rules compile through [`fancy_regex`], which delegates a pattern to
//! the linear-time [`regex`](https://docs.rs/regex) crate whenever it
//! contains no backreference or lookaround, and only falls back to its
//! own backtracking engine for the rest.
//!
//! That fallback is not optional. [`regex`](https://docs.rs/regex), like
//! Go's RE2-based `regexp`, deliberately excludes backreferences - and
//! at least one real locale file needs one *inside a matching pattern*:
//! `hu.sor` encodes a grammatical-suffix "database" as a single string
//! and looks entries up with `.*:\1:(-[^-:]*).*`, where `\1` correlates
//! the requested case name with its entry. The reference C++/Java/Python
//! implementations get this for free from their native backtracking
//! engines.
//!
//! Every backtracking match is bounded by
//! [`BACKTRACK_LIMIT`], so a pathological
//! pattern degrades to "this rule doesn't match" instead of hanging -
//! the standard mitigation for catastrophic backtracking (`ReDoS`), and
//! the reason the fallback stays scoped to the patterns that actually
//! need it: every other rule keeps the linear-time guarantee.
//!
//! ## Two designs considered and rejected
//!
//! **Gluing two engines together by hand** - trying
//! [`regex`](https://docs.rs/regex) first and falling back to a separate
//! backtracking crate on a compile error - is what an equivalent Go port
//! has to do, because Go has no single crate covering both. Here it
//! would be strictly worse: [`fancy_regex`] already makes the same
//! decision internally, per pattern, from the parsed AST rather than
//! from whether a compile call happened to fail, and one engine means
//! one dialect. Two engines mean two subtly different ones, and a rule
//! silently changing meaning depending on which side it landed on.
//!
//! **Compiling a "probe" under the linear engine** - replacing the
//! backreference with a copy of the referenced group's own pattern, then
//! verifying group equality after the fact - avoids the backtracking
//! dependency entirely and works for every backreference in the bundled
//! data today. It is also provably incomplete: a linear-engine match
//! picks one split deterministically, so for a pattern like `(a+)\1`
//! against a string with more than one valid split, the chosen candidate
//! can fail the equality check even though a valid split exists. A real
//! backtracking engine finds it; the probe reports a false negative.
//! `tests/regex.rs` pins that exact case.

// Compiles and runs this crate's README example as a doctest, so it is
// verified rather than merely plausible. See the same pattern in the
// `numbertext` crate.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeExamples;

#[cfg(doctest)]
#[doc = include_str!("../README.pt-BR.md")]
struct ExemplosDoLeiame;

mod escape;
mod eval;
mod lexer;
mod program;
mod regex;
mod template;

pub use eval::MAX_RECURSION_DEPTH;
pub use lexer::language_tags;
pub use program::{Program, SkippedRule};
pub use regex::BACKTRACK_LIMIT;
pub use template::TemplateError;
