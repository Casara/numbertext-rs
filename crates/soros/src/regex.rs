//! Pattern compilation.
//!
//! See the crate-level docs for why a single [`fancy_regex`] engine
//! replaces the two hand-glued engines an equivalent Go port needs.

use fancy_regex::{Captures, Regex, RegexBuilder};

/// Capture groups from a match against a `&str`.
///
/// [`fancy_regex`] parameterizes [`Captures`] over the haystack type;
/// every pattern here is matched against string input, so this alias
/// pins that once instead of threading the parameter through the
/// evaluator.
pub(crate) type StrCaptures<'t> = Captures<'t, str>;

/// Bounds the work a single match attempt may spend backtracking.
///
/// This only ever applies to patterns [`fancy_regex`] cannot delegate to
/// the linear-time [`regex`](https://docs.rs/regex) engine - the rare
/// ones using a backreference or lookaround. It is a safety net against
/// a pathological pattern's catastrophic backtracking, not a budget the
/// ordinary path ever approaches.
///
/// Being a *step count* rather than a wall-clock timeout makes it
/// deterministic: the same pattern and input either fit the budget or
/// don't, identically on every machine. A time-based bound - what a Go
/// port has to use, since `regexp2` only offers `MatchTimeout` - makes
/// the same call succeed on a fast machine and fail on a loaded CI
/// runner, which is a far worse property for a library whose output is
/// meant to be reproducible.
///
/// The value is deliberately generous: every bundled locale's real
/// patterns resolve in a handful of steps, and the shapes that reach a
/// million are already exponential (see `tests/regex.rs`).
pub const BACKTRACK_LIMIT: usize = 1_000_000;

/// A compiled rule pattern: the one capability a rule needs, which is to
/// try a full-string match and report capture groups if it succeeded.
///
/// [`fancy_regex`] inspects the pattern and delegates it wholesale to
/// the linear-time [`regex`](https://docs.rs/regex) crate whenever it
/// contains no backreference or lookaround, which is every rule but a
/// handful across all the bundled locale files. Only the rest run on its
/// backtracking engine, bounded by [`BACKTRACK_LIMIT`].
#[derive(Debug)]
pub(crate) struct Matcher {
    re: Regex,
}

impl Matcher {
    /// Runs the pattern against `text`.
    ///
    /// A match-time error - in practice, [`BACKTRACK_LIMIT`] being
    /// exhausted - is reported as "no match" rather than propagated.
    /// That is the same outcome a rule with no matching input already
    /// has, and the correct degrade for an exhausted backtracking
    /// budget: the rule simply doesn't fire, and the next one gets its
    /// turn.
    pub(crate) fn captures<'t>(&self, text: &'t str) -> Option<StrCaptures<'t>> {
        self.re.captures(text).ok().flatten()
    }
}

/// Compiles a full-match pattern, already wrapped in `^(?:...)$` by the
/// caller.
///
/// # Errors
///
/// Returns the underlying [`fancy_regex::Error`] if the pattern is not
/// valid regex syntax. The caller records it against the offending rule
/// rather than failing the whole program (see
/// [`crate::SkippedRule`]).
pub(crate) fn compile_pattern(wrapped: &str) -> Result<Matcher, fancy_regex::Error> {
    let re = RegexBuilder::new(wrapped)
        .backtrack_limit(BACKTRACK_LIMIT)
        .build()?;

    Ok(Matcher { re })
}
