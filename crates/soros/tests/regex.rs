//! The regex engine's two guarantees: backreferences work, and
//! backtracking is bounded.
//!
//! These go through `Program` rather than the internal matcher on
//! purpose - what matters is the behavior a rule file actually gets, not
//! the shape of the private API underneath it.

use std::time::{Duration, Instant};

use soros::Program;

/// A backreference *inside a matching pattern* is the one thing the
/// linear-time [`regex`](https://docs.rs/regex) engine cannot express,
/// and at least one bundled locale genuinely needs it: `hu.sor` looks
/// grammatical-suffix entries up out of a single packed string with
/// `.*:\1:(-[^-:]*).*`.
#[test]
fn backreference_in_a_pattern_matches() {
    let program = Program::compile(r"([a-z]+)\1 doubled-\1", "");

    assert_eq!(program.run("abab"), "doubled-ab");
    assert_eq!(program.run("abc"), "");
    assert!(program.skipped().is_empty(), "the pattern must compile");
}

/// The case that rules out the cheaper design: matching a backreference
/// by compiling a "probe" under the linear engine (the backreference
/// position replaced by a copy of the referenced group's pattern) and
/// verifying group equality afterwards.
///
/// `(a+)\1` against "aaaa" has more than one way to split. A linear
/// engine's single deterministic candidate is the greedy one - `\1` =
/// "aaaa", leaving nothing to repeat - which fails the equality check
/// even though the valid split `\1` = "aa" exists. A real backtracking
/// engine finds it, and so must this crate: a false negative here is a
/// rule silently not firing.
#[test]
fn backreference_finds_a_non_greedy_split() {
    let program = Program::compile(r"(a+)\1 half-is-\1", "");

    assert_eq!(program.run("aaaa"), "half-is-aa");
}

/// Lookaround is the other construct the linear engine excludes.
#[test]
fn lookaround_in_a_pattern_matches() {
    let program = Program::compile(r"(?!foo)\w+ not-foo", "");

    assert_eq!(program.run("bar"), "not-foo");
    assert_eq!(program.run("foo"), "");
}

/// A pattern with no backreference or lookaround is delegated to the
/// linear-time engine, so a shape that is catastrophic for a backtracker
/// costs nothing.
///
/// `(a+)+b` against a long run of `a` with no `b` is the textbook ReDoS
/// case: exponential under backtracking, trivial under an automaton.
/// Measured here, delegation resolves it in microseconds; the same input
/// forced through the backtracker exhausts `BACKTRACK_LIMIT` in tens of
/// milliseconds. The budget below is orders of magnitude above both, so
/// this asserts the property without being a timing-sensitive test - it
/// fails only if the pattern is genuinely being explored exponentially.
#[test]
fn backreference_free_patterns_take_the_linear_path() {
    let program = Program::compile("(a+)+b matched", "");
    let haystack = "a".repeat(10_000);

    let started = Instant::now();
    let got = program.run(&haystack);
    let elapsed = started.elapsed();

    assert_eq!(got, "", "no 'b', so the rule must not match");
    assert!(
        elapsed < Duration::from_secs(5),
        "a delegated pattern took {elapsed:?} - it is being backtracked"
    );
}

/// A pattern that genuinely does backtrack is bounded: exceeding the
/// budget degrades to "this rule doesn't match", which lets the next
/// rule take over, rather than hanging the caller.
///
/// `(a|a)+\1` cannot be delegated (it has a backreference) and every
/// alternation choice has to be explored, so it blows past
/// `BACKTRACK_LIMIT` well before this input length.
#[test]
fn catastrophic_backtracking_degrades_to_no_match() {
    let program = Program::compile("(a|a)+\\1 matched\n.* fell-through", "");
    let haystack = format!("{}X", "a".repeat(30));

    let started = Instant::now();
    let got = program.run(&haystack);

    assert_eq!(
        got, "fell-through",
        "the bounded rule must give up and let the next rule match"
    );
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "the backtracking bound did not fire"
    );
}

/// An invalid pattern is reported against its own rule, not raised as a
/// failure of the whole program.
#[test]
fn an_invalid_pattern_is_reported_per_rule() {
    let program = Program::compile("(unbalanced whatever\n1 one\n", "");

    assert_eq!(program.skipped().len(), 1);
    assert!(program.skipped()[0].reason.contains("invalid pattern"));
    assert_eq!(program.run("1"), "one");
}
