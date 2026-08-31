//! Hiding and revealing the spec's backslash-escaped special characters.

/// Soros special characters (spec 2.6) that must be quoted with a
/// backslash to be used literally: `\\`, `\"`, `\$`, `\(`, `\)`, `\[`,
/// `\]`, `\|`, `\#`, `\;` and the `\n` newline notation. Each is mapped
/// to a private-use-area character while the template/pattern text is
/// scanned for real syntax, then mapped back to the literal character
/// once a piece of text is known to be plain output (see
/// [`reveal_escapes`]).
///
/// Private-use code points are the right carrier here precisely because
/// they cannot occur in a real `.sor` file's text: UTF-8 encodes them as
/// three bytes that never collide with the ASCII syntax characters the
/// scanner looks for.
pub(crate) const PH_BACKSLASH: char = '\u{E000}';
pub(crate) const PH_QUOTE: char = '\u{E001}';
pub(crate) const PH_DOLLAR: char = '\u{E002}';
pub(crate) const PH_LPAREN: char = '\u{E003}';
pub(crate) const PH_RPAREN: char = '\u{E004}';
pub(crate) const PH_LBRACKET: char = '\u{E005}';
pub(crate) const PH_RBRACKET: char = '\u{E006}';
pub(crate) const PH_PIPE: char = '\u{E007}';
pub(crate) const PH_HASH: char = '\u{E008}';
pub(crate) const PH_SEMICOLON: char = '\u{E009}';
pub(crate) const PH_NEWLINE: char = '\u{E00A}';

/// Maps the character following a backslash to its placeholder.
fn escape_to_placeholder(c: char) -> Option<char> {
    Some(match c {
        '\\' => PH_BACKSLASH,
        '"' => PH_QUOTE,
        '$' => PH_DOLLAR,
        '(' => PH_LPAREN,
        ')' => PH_RPAREN,
        '[' => PH_LBRACKET,
        ']' => PH_RBRACKET,
        '|' => PH_PIPE,
        '#' => PH_HASH,
        ';' => PH_SEMICOLON,
        'n' => PH_NEWLINE,
        _ => return None,
    })
}

/// Maps a placeholder back to its literal character, for text that is
/// known to be plain output.
fn placeholder_to_literal(c: char) -> Option<char> {
    Some(match c {
        PH_BACKSLASH => '\\',
        PH_QUOTE => '"',
        PH_DOLLAR => '$',
        PH_LPAREN => '(',
        PH_RPAREN => ')',
        PH_LBRACKET => '[',
        PH_RBRACKET => ']',
        PH_PIPE => '|',
        PH_HASH => '#',
        PH_SEMICOLON => ';',
        PH_NEWLINE => '\n',
        _ => return None,
    })
}

/// Maps a placeholder back to a regex-metacharacter-safe literal
/// representation, used when compiling a pattern (as opposed to
/// rendering template output, see [`placeholder_to_literal`]).
fn placeholder_to_regex(c: char) -> Option<&'static str> {
    Some(match c {
        PH_BACKSLASH => r"\\",
        PH_QUOTE => "\"",
        PH_DOLLAR => r"\$",
        PH_LPAREN => r"\(",
        PH_RPAREN => r"\)",
        PH_LBRACKET => r"\[",
        PH_RBRACKET => r"\]",
        PH_PIPE => r"\|",
        PH_HASH => "#",
        PH_SEMICOLON => ";",
        PH_NEWLINE => "\n",
        _ => return None,
    })
}

/// Replaces every recognized `\X` escape sequence in `s` with a
/// private-use placeholder character, so the Soros syntax scanner (the
/// template parser, the pattern lexer) never mistakes an escaped
/// character for real syntax.
///
/// A backslash followed by an ASCII digit is left untouched: that is
/// always a backreference (`\1`..`\9`, or `\0` for the whole match),
/// never an escape sequence. No special case is needed for it here,
/// since a digit is never an [`escape_to_placeholder`] key and so
/// already falls through unmodified.
pub(crate) fn hide_escapes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(ph) = chars.peek().copied().and_then(escape_to_placeholder) {
                out.push(ph);
                chars.next();
                continue;
            }
        }
        out.push(c);
    }

    out
}

/// Maps placeholders produced by [`hide_escapes`] back to their literal
/// characters. Applied only to text already known to be plain output (a
/// [`crate::template::Node::Literal`]), once syntax scanning is
/// complete.
pub(crate) fn reveal_escapes(s: &str) -> String {
    s.chars()
        .map(|c| placeholder_to_literal(c).unwrap_or(c))
        .collect()
}

/// Maps placeholders back to their literal, regex-metacharacter-safe
/// form so the result can be embedded in a pattern string handed to the
/// regex compiler.
pub(crate) fn reveal_escapes_for_regex(s: &str) -> String {
    let mut out = String::with_capacity(s.len());

    for c in s.chars() {
        match placeholder_to_regex(c) {
            Some(lit) => out.push_str(lit),
            None => out.push(c),
        }
    }

    out
}
