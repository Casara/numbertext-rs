//! Parsing a rule's replacement text into an AST.

use std::fmt;

use crate::escape::reveal_escapes;

/// One element of a parsed replacement template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Node {
    /// Plain text, emitted as-is.
    Literal(String),
    /// A raw `\N` reference: the captured text of group `N` is
    /// substituted verbatim, without being re-run through the engine.
    Backref(usize),
    /// A recursive call, from either the abbreviated `$N` form or the
    /// full `$(...)` form. `arg` is the (already parsed) template used
    /// to build the call's argument string before recursing.
    Call {
        arg: Vec<Node>,
        pipe_before: bool,
        pipe_after: bool,
    },
    /// A `[...]` conditional block: its rendered text is kept only if
    /// every [`Node::Call`] directly inside it (not nested inside
    /// another [`Node::Optional`]) evaluates to a non-empty string.
    Optional(Vec<Node>),
    /// A boundary-modifier marker (`|`). It never produces output and
    /// never survives parsing: [`attach_pipes`] folds it into the
    /// `pipe_before`/`pipe_after` flags of the neighboring calls and
    /// drops it.
    Pipe,
}

/// A replacement template that could not be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateError {
    /// A `[` conditional block was never closed.
    UnterminatedBracket,
    /// A `$(` call was never closed.
    UnterminatedCall,
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnterminatedBracket => f.write_str("unterminated '[' in template"),
            Self::UnterminatedCall => f.write_str("unterminated '$(' in template"),
        }
    }
}

impl std::error::Error for TemplateError {}

/// Parses a rule's replacement text (with escapes already hidden, see
/// [`crate::escape::hide_escapes`]) into a sequence of nodes.
///
/// # Errors
///
/// Returns [`TemplateError`] if a `[` or `$(` is never closed.
pub(crate) fn parse_template(s: &str) -> Result<Vec<Node>, TemplateError> {
    let chars: Vec<char> = s.chars().collect();
    // `inside_optional` is false here, so on success this always
    // consumes all of `chars`: the only early-return-with-leftover path
    // requires an enclosing '['.
    let (nodes, _) = parse_nodes(&chars, false)?;

    Ok(attach_pipes(&nodes))
}

/// Parses a run of nodes until either the input is exhausted or, when
/// `inside_optional` is true, a closing `]` is found.
///
/// Returns the parsed nodes and the index parsing stopped at (pointing
/// at the `]` when it stopped early). One match arm per Soros token kind
/// (spec 2.4): splitting the dispatch itself - as opposed to each arm's
/// body, already extracted into [`parse_bracket`]/[`parse_call`] - would
/// trade a single readable state machine for several that have to be
/// read together to see the whole grammar.
fn parse_nodes(s: &[char], inside_optional: bool) -> Result<(Vec<Node>, usize), TemplateError> {
    // Emits the literal text accumulated so far, if any. Escapes are
    // revealed here, at the point the text is known to be plain output.
    fn flush(lit: &mut String, nodes: &mut Vec<Node>) {
        if !lit.is_empty() {
            nodes.push(Node::Literal(reveal_escapes(lit)));
            lit.clear();
        }
    }

    let mut nodes: Vec<Node> = Vec::new();
    let mut lit = String::new();
    let mut i = 0;

    while i < s.len() {
        let c = s[i];
        let next = s.get(i + 1).copied();

        match c {
            ']' if inside_optional => {
                flush(&mut lit, &mut nodes);
                return Ok((nodes, i));
            }
            '[' => {
                flush(&mut lit, &mut nodes);
                let (node, consumed) = parse_bracket(&s[i + 1..])?;
                nodes.push(node);
                i += 1 + consumed;
            }
            '|' => {
                flush(&mut lit, &mut nodes);
                nodes.push(Node::Pipe);
                i += 1;
            }
            '\\' if next.is_some_and(|n| n.is_ascii_digit()) => {
                flush(&mut lit, &mut nodes);
                nodes.push(Node::Backref(digit(next)));
                i += 2;
            }
            // "$N" is sugar for "$(\N)" (spec 2.4.2). Only 1-9: "$0"
            // is not a call form.
            '$' if next.is_some_and(|n| n.is_ascii_digit() && n != '0') => {
                flush(&mut lit, &mut nodes);
                nodes.push(Node::Call {
                    arg: vec![Node::Backref(digit(next))],
                    pipe_before: false,
                    pipe_after: false,
                });
                i += 2;
            }
            '$' if next == Some('(') => {
                flush(&mut lit, &mut nodes);
                let (node, consumed) = parse_call(&s[i + 2..])?;
                nodes.push(node);
                i += 2 + consumed;
            }
            _ => {
                lit.push(c);
                i += 1;
            }
        }
    }

    flush(&mut lit, &mut nodes);

    Ok((nodes, i))
}

/// Converts a known-ASCII-digit character to its numeric value.
fn digit(c: Option<char>) -> usize {
    c.and_then(|c| c.to_digit(10)).unwrap_or(0) as usize
}

/// Parses the content of a `[...]` conditional block whose opening `[`
/// was already consumed. Returns the node and how many characters of
/// `s` it used, including the closing `]`.
fn parse_bracket(s: &[char]) -> Result<(Node, usize), TemplateError> {
    let (children, stopped_at) = parse_nodes(s, true)?;

    if s.get(stopped_at) != Some(&']') {
        return Err(TemplateError::UnterminatedBracket);
    }

    Ok((Node::Optional(children), stopped_at + 1))
}

/// Parses a full-form `$(...)` call whose leading `$(` was already
/// consumed. Returns the node and how many characters of `s` it used,
/// including the closing `)`.
fn parse_call(s: &[char]) -> Result<(Node, usize), TemplateError> {
    let inner_end = scan_parens(s)?;
    // A call argument may itself contain balanced, unescaped
    // parentheses belonging to nested "$(...)" calls, which
    // `scan_parens` has already accounted for. Parsing it with
    // `inside_optional` false means it is always fully consumed.
    let (arg, _) = parse_nodes(&s[..inner_end], false)?;

    Ok((
        Node::Call {
            arg,
            pipe_before: false,
            pipe_after: false,
        },
        inner_end + 1,
    ))
}

/// Finds the index of the matching `)` for a `$(` that was already
/// consumed, honoring nested parentheses belonging to inner `$(...)`
/// calls.
fn scan_parens(s: &[char]) -> Result<usize, TemplateError> {
    let mut depth = 1_usize;

    for (i, &c) in s.iter().enumerate() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(i);
                }
            }
            _ => {}
        }
    }

    Err(TemplateError::UnterminatedCall)
}

/// Folds [`Node::Pipe`] markers into the `pipe_before`/`pipe_after`
/// flags of neighboring calls and removes them, since a pipe by itself
/// never renders anything.
///
/// A single pipe between two calls applies to both (spec 2.4.4: "`$1|$2`
/// is equivalent form of `$1||$2`").
fn attach_pipes(nodes: &[Node]) -> Vec<Node> {
    let is_pipe = |n: Option<&Node>| matches!(n, Some(Node::Pipe));

    let mut out: Vec<Node> = Vec::with_capacity(nodes.len());

    for (i, node) in nodes.iter().enumerate() {
        match node {
            Node::Pipe => {}
            Node::Call { arg, .. } => out.push(Node::Call {
                arg: attach_pipes(arg),
                pipe_before: i > 0 && is_pipe(nodes.get(i - 1)),
                pipe_after: is_pipe(nodes.get(i + 1)),
            }),
            Node::Optional(children) => out.push(Node::Optional(attach_pipes(children))),
            other => out.push(other.clone()),
        }
    }

    out
}
