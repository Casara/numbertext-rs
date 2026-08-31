//! The Soros specification's own canonical examples, plus targeted
//! boundary/bracket/macro cases.
//!
//! These come first when changing the engine: they exercise the language
//! itself, with no dependency on any real locale file. If they pass but
//! a bundled `.sor` file misbehaves, that means the spec has a subtlety
//! not covered here yet - add a minimal reproduction rather than
//! debugging against a 300-line locale.

use soros::Program;

/// Specification example 1 (section 3): a program with no
/// NUMBERTEXT-mode concerns, since it uses no digits - but the hidden
/// left-zero rule is always present and must simply never match
/// non-numeric input.
#[test]
fn reverses_a_string() {
    let program = Program::compile(r"(.*)(.) \2$1", "");

    assert_eq!(program.run("hello"), "olleh");
}

/// Specification example 2.
#[test]
fn inserts_thousands_separators() {
    let program = Program::compile("(\\d+)(\\d{3}) $1,\\2\n(\\d+) \\1\n", "");

    for (input, want) in [
        ("1", "1"),
        ("999", "999"),
        ("1000", "1,000"),
        ("1234567", "1,234,567"),
        ("123456789", "123,456,789"),
    ] {
        assert_eq!(program.run(input), want, "input {input}");
    }
}

/// Specification example 3.
#[test]
fn converts_to_devanagari_digits() {
    let source = "\
(\\d*)0 $1०
(\\d*)1 $1१
(\\d*)2 $1२
(\\d*)3 $1३
(\\d*)4 $1४
(\\d*)5 $1५
(\\d*)6 $1६
(\\d*)7 $1७
(\\d*)8 $1८
(\\d*)9 $1९
";
    let program = Program::compile(source, "");

    assert_eq!(program.run("0"), "०");
    assert_eq!(program.run("29"), "२९");
}

/// Specification example 4: the small English number-name program shown
/// inline in the spec text, distinct from the full `en.sor` the
/// `numbertext` crate bundles.
#[test]
fn spells_out_english_number_names() {
    let source = "\
__numbertext__
^0 zero; 1 one; 2 two; 3 three; 4 four; 5 five;
6 six; 7 seven; 8 eight; 9 nine; 10 ten; 11 eleven; 12 twelve;
13 thirteen; 15 fifteen; 18 eighteen; 1(\\d) $1teen
2(\\d) twenty[-$1]; 3(\\d) thirty[-$1]; 4(\\d) forty[-$1]
5(\\d) fifty[-$1]; 8(\\d) eighty[-$1]
(\\d)(\\d) $1ty[-$2]
(\\d)(\\d\\d) $1 hundred[ and $2]
";
    let program = Program::compile(source, "");

    for (input, want) in [
        ("0", "zero"),
        ("1", "one"),
        ("9", "nine"),
        ("10", "ten"),
        ("13", "thirteen"),
        ("20", "twenty"),
        ("21", "twenty-one"),
        ("42", "forty-two"),
        ("99", "ninety-nine"),
        ("100", "one hundred"),
        ("101", "one hundred and one"),
        // The hidden left-zero-deletion rule (spec 2.5), placed by the
        // explicit __numbertext__ directive above.
        ("007", "seven"),
        ("00", "zero"),
    ] {
        assert_eq!(program.run(input), want, "input {input}");
    }
}

/// A `[...]` block disappears entirely - bracket text and all - when the
/// call inside it is empty, and is kept with the call substituted when
/// it is not (spec 2.4.5).
#[test]
fn conditional_text_drops_when_its_call_is_empty() {
    let program = Program::compile("1 one\n2 two\n(\\d)(\\d) $1[-$2]\n0\n", "");

    assert_eq!(program.run("10"), "one");
    assert_eq!(program.run("12"), "one-two");
}

/// A `|` immediately before or after a call forces its
/// leading/trailing-boundary eligibility regardless of where the call
/// sits in the template (spec 2.4.4) - the mechanism real locale files
/// use for negative numbers, as `negative |$1`.
#[test]
fn pipe_forces_boundary_eligibility() {
    let program = Program::compile("^0 zero\n[-−](\\d+) negative |$1", "");

    assert_eq!(program.run("-0"), "negative zero");
}

/// A pattern ending in `$` (spec 2.3.2's trailing-boundary anchor) only
/// applies when the current call is itself at the trailing boundary of
/// the template it is embedded in, so the same digit takes two different
/// rules depending on its position.
#[test]
fn trailing_anchor_only_fires_at_the_trailing_boundary() {
    let program = Program::compile("1 one\n2$ two-at-end\n2 two\n(\\d)(\\d) $1 $2\n", "");

    assert_eq!(program.run("22"), "two two-at-end");
}

/// A call inside a `[...]` block inherits the enclosing begin/end
/// boundary only when it is actually the first/last child of the bracket
/// *and* the bracket itself sits at that boundary of its own template
/// (spec 2.4.4: brackets are inline, they do not start a new template).
#[test]
fn optional_block_propagates_boundaries_positionally() {
    let program = Program::compile(
        "a alpha\n^z here-begin\nz$ here-end\nz neither\n(a)(z) $1[x$2]\n",
        "",
    );

    // The outer call is "az", which does not itself match "^z"/"z$"
    // (full-match), so the boundary status of "z" can only come from how
    // (a)(z)'s template propagates it through the bracket.
    //
    // $2 ("z") is the bracket's *second* child, so it must not inherit
    // begin=true even though the bracket is the template's last node -
    // "^z here-begin" must not fire. It is the bracket's last child, so
    // it does inherit end=true, and "z$ here-end" wins over the
    // unanchored "z neither".
    assert_eq!(program.run("az"), "alphaxhere-end");
}

/// Referencing a capture group the matched pattern has no group for
/// yields an empty string rather than panicking.
#[test]
fn out_of_range_backreference_is_empty() {
    let program = Program::compile(r"(\d) \2", "");

    assert_eq!(program.run("5"), "");
}

/// `\0` is the whole match, and a valid group index - not rejected the
/// way a negative one would be. Real locale files use it: `pt.sor` and
/// `hu.sor` both build `help` output with `$(\0 feminine)`-style calls.
#[test]
fn group_zero_is_the_whole_match() {
    let program = Program::compile(r"(\d)(\d) \0", "");

    assert_eq!(program.run("12"), "12");
}

/// `== name ==` sections (spec 2.8): calling a prefixed rule set from
/// outside by constructing `"<prefix> <arg>"`, including
/// cardinal-to-ordinal ending rewriting through it.
#[test]
fn macro_prefix_sections() {
    let source = "\
1 one
2 two
3 three

== ordinal ==
(\\d+) $(ordinal |$1)
(.*)one \\1first
(.*)two \\1second
(.*)three \\1third
";
    let program = Program::compile(source, "");

    assert_eq!(program.run("ordinal 1"), "first");
    assert_eq!(program.run("ordinal 2"), "second");
    assert_eq!(program.run("ordinal 3"), "third");
}

/// `== ==` turns macro expansion back off for the lines after it
/// (spec 2.8).
#[test]
fn empty_macro_directive_disables_the_prefix() {
    let program = Program::compile("== ordinal ==\n1 first\n== ==\n2 two\n", "");

    assert_eq!(program.run("ordinal 1"), "first");
    assert_eq!(program.run("2"), "two");
}

/// `[:lang-code:]` comment tags (spec 2.7.1): a tagged line applies only
/// for the requested language, otherwise the next untagged rule takes
/// over.
#[test]
fn language_conditional_lines() {
    let source = "(\\d)(\\d\\d) \\1 hundred and \\2 # [:en-GB:]\n(\\d)(\\d\\d) \\1 hundred \\2\n";

    assert_eq!(
        Program::compile(source, "en-GB").run("123"),
        "1 hundred and 23"
    );
    assert_eq!(Program::compile(source, "en-US").run("123"), "1 hundred 23");
}

/// A statement whose pattern or template is malformed is recorded rather
/// than taken as a reason to fail the whole program: every other rule
/// keeps working.
#[test]
fn a_malformed_rule_is_skipped_not_fatal() {
    let program = Program::compile("1 one\n(unclosed two\n2 two\n", "");

    assert_eq!(program.run("1"), "one");
    assert_eq!(program.run("2"), "two");
    assert_eq!(program.skipped().len(), 1);
    assert!(program.skipped()[0].statement.contains("unclosed"));
}

/// Recursion is bounded, so a rule file that recurses without converging
/// yields an empty string instead of overflowing the stack and aborting
/// the process (see `soros::MAX_RECURSION_DEPTH`).
#[test]
fn runaway_recursion_is_bounded() {
    // "x" rewrites to "x" forever.
    let program = Program::compile("x $(x)", "");

    assert_eq!(program.run("x"), "");
}

/// A `[...]` block is kept or dropped on the strength of its **first**
/// call. A later call inside it may be empty without discarding the
/// block.
///
/// This is not a free reading of spec 2.4.5. The reference
/// implementations rewrite `[before$CALL after]` into
/// `$(SEP before SEP $CALL SEP after)` textually and let two hidden
/// rules decide - `SEP(.*)SEP(.+)SEP(.*)` keeps, `SEP.*SEPSEP.*` drops.
/// Only one call ever reaches that middle slot, so only one can decide.
///
/// Verified against upstream's own C++ engine, which produces
/// `Xpone q` and `Y` for these two inputs.
#[test]
fn conditional_text_is_decided_by_its_first_call() {
    let program = Program::compile("a one\nz\n(a)(z) X[p$1 q$2]\n(z)(a) Y[p$1 q$2]\n", "");

    // First call non-empty ("one"), second empty: kept.
    assert_eq!(program.run("az"), "Xpone q");
    // First call empty: dropped, whatever follows it.
    assert_eq!(program.run("za"), "Y");
}
