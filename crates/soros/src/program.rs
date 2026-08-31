//! Compiling source into an ordered rule list.

use std::fmt;

use crate::escape::{hide_escapes, reveal_escapes_for_regex};
use crate::lexer::{
    macro_prefix, split_pattern_value, split_statements, strip_comments_and_filter_lang,
};
use crate::regex::{Matcher, compile_pattern};
use crate::template::{Node, parse_template};

/// The bare statement marking where the default NUMBERTEXT
/// left-zero-deletion rule is inserted (spec 2.5). When absent, the rule
/// goes at the very start of the program (default mode).
const NUMBERTEXT_DIRECTIVE: &str = "__numbertext__";

/// The hidden rule every Soros program gets in NUMBERTEXT mode,
/// stripping extra leading zeros - also after an optional macro-prefix
/// word, so it keeps working inside `== prefix ==` sections. The pattern
/// is quoted because it contains a literal space (spec 2.3.3).
const DEFAULT_LEFT_ZERO_RULE: &str = r#""([a-z][-a-z]* )?0+(0|[1-9]\d*)" $(\1\2)"#;

/// One compiled program line.
pub(crate) struct Rule {
    pub(crate) re: Matcher,
    pub(crate) tmpl: Vec<Node>,
    /// The original pattern carried an explicit leading `^` anchor.
    pub(crate) begin: bool,
    /// The original pattern carried an explicit trailing `$` anchor.
    pub(crate) end: bool,
}

/// A program line [`Program::compile`] could not use: either its pattern
/// is not valid regex syntax, or its replacement template doesn't parse.
///
/// Rather than fail a whole locale over one bad line, compilation
/// records it here and moves on: the rule simply never matches, and
/// every other rule keeps working. A rule that cannot be parsed is
/// reported, never guessed at.
#[derive(Debug, Clone)]
pub struct SkippedRule {
    /// The original statement text, as written in the source.
    pub statement: String,
    /// Why it was skipped.
    pub reason: String,
}

impl fmt::Display for SkippedRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.reason, self.statement)
    }
}

/// A compiled Soros program: an ordered list of rules plus the single
/// recursive entry point they implicitly define (the `$` function of the
/// spec).
///
/// See the [crate-level documentation](crate) for the language itself
/// and a worked example.
pub struct Program {
    pub(crate) rules: Vec<Rule>,
    skipped: Vec<SkippedRule>,
}

impl fmt::Debug for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Program")
            .field("rules", &self.rules.len())
            .field("skipped", &self.skipped)
            .finish()
    }
}

impl Program {
    /// Compiles Soros source code into a runnable program for the given
    /// language code.
    ///
    /// `lang` selects which `[:lang-code:]` conditional lines apply
    /// (spec 2.7.1); pass `""` if the source has none.
    ///
    /// This is infallible by design. There is no such thing as a source
    /// that fails to tokenize - every failure is per-rule and is
    /// recorded in [`Program::skipped`] instead, so one malformed line
    /// never takes a whole locale down with it.
    ///
    /// ```
    /// let program = soros::Program::compile("1 one\n2 two\n", "");
    ///
    /// assert_eq!(program.run("2"), "two");
    /// assert!(program.skipped().is_empty());
    /// ```
    #[must_use]
    pub fn compile(source: &str, lang: &str) -> Self {
        let hidden = hide_escapes(source);
        let code_only = strip_comments_and_filter_lang(&hidden, lang);
        let statements = inject_default_left_zero_rule(split_statements(&code_only));

        let mut rules = Vec::new();
        let mut skipped = Vec::new();
        let mut prefix: Option<String> = None;

        for stmt in statements {
            if let Some(p) = macro_prefix(&stmt) {
                // "== ==" (an empty prefix) turns macro expansion back
                // off for the statements that follow (spec 2.8).
                prefix = if p.is_empty() { None } else { Some(p) };
                continue;
            }

            let (mut pattern, value) = split_pattern_value(&stmt);
            if let Some(p) = &prefix {
                pattern = apply_prefix(p, &pattern);
            }

            let begin = pattern.starts_with('^');
            if begin {
                pattern.remove(0);
            }
            let end = pattern.ends_with('$');
            if end {
                pattern.pop();
            }

            let body = reveal_escapes_for_regex(&pattern);
            let re = match compile_pattern(&format!("^(?:{body})$")) {
                Ok(re) => re,
                Err(err) => {
                    skipped.push(SkippedRule {
                        statement: stmt,
                        reason: format!("invalid pattern: {err}"),
                    });
                    continue;
                }
            };

            let tmpl = match parse_template(&value) {
                Ok(tmpl) => tmpl,
                Err(err) => {
                    skipped.push(SkippedRule {
                        statement: stmt,
                        reason: format!("invalid replacement: {err}"),
                    });
                    continue;
                }
            };

            rules.push(Rule {
                re,
                tmpl,
                begin,
                end,
            });
        }

        Self { rules, skipped }
    }

    /// The program lines that could not be compiled, if any.
    ///
    /// An empty slice means every statement in the source became a
    /// usable rule.
    #[must_use]
    pub fn skipped(&self) -> &[SkippedRule] {
        &self.skipped
    }

    /// How many usable rules the program has.
    #[must_use]
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

/// Concatenates a macro prefix in front of a rule's pattern (spec 2.8),
/// hoisting a leading `^` boundary anchor in front of the prefix when
/// present, and using the prefix alone when the rule's own pattern is
/// empty.
fn apply_prefix(prefix: &str, pattern: &str) -> String {
    if pattern.is_empty() {
        return prefix.to_owned();
    }

    match pattern.strip_prefix('^') {
        Some(rest) => format!("^{prefix} {rest}"),
        None => format!("{prefix} {pattern}"),
    }
}

/// Inserts the hidden left-zero-deletion rule (spec 2.5) at the position
/// of an explicit `__numbertext__` directive, or at the very start of
/// the program when there is none.
fn inject_default_left_zero_rule(mut statements: Vec<String>) -> Vec<String> {
    let rule = DEFAULT_LEFT_ZERO_RULE.to_owned();

    match statements.iter().position(|s| s == NUMBERTEXT_DIRECTIVE) {
        Some(i) => statements[i] = rule,
        None => statements.insert(0, rule),
    }

    statements
}
