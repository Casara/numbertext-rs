//! Turning raw `.sor` source into statements: comment stripping,
//! `[:lang:]` filtering, statement splitting, and the pattern/value
//! split.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use fancy_regex::Regex;

// Both patterns below are compile-time constants written in this file,
// so a compile failure is a bug here rather than bad input - hence the
// `expect`.

/// Matches a `[:lang-code:]` tag inside a comment (spec 2.7.1).
static LANG_TAG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[:([^:\]]+):\]").expect("built-in lang-tag pattern must compile")
});

/// Matches a `== prefix-literal ==` statement (spec 2.8).
static MACRO_DIRECTIVE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^==\s*(.*?)\s*==$").expect("built-in macro-directive pattern must compile")
});

/// Removes `#` comments from every physical line of `hidden` (comments
/// run to the end of the physical line, spec 2.7) and blanks out any
/// line whose comment names one or more `[:lang:]` tags that do not
/// include `lang` (spec 2.7.1).
///
/// Escaped characters must already be hidden (see
/// [`crate::escape::hide_escapes`]) so that a remaining `#` is
/// unambiguously a comment marker.
///
/// A filtered-out line becomes an empty line rather than disappearing,
/// so that line-oriented reasoning about the source stays stable.
pub(crate) fn strip_comments_and_filter_lang(hidden: &str, lang: &str) -> String {
    let lang = lang.trim().to_lowercase();
    let mut kept: Vec<&str> = Vec::new();

    for line in hidden.split('\n') {
        let (code, comment) = match line.split_once('#') {
            Some((code, comment)) => (code, comment),
            None => (line, ""),
        };

        let mut tags = LANG_TAG_RE.captures_iter(comment).peekable();
        if tags.peek().is_some() {
            let matched = tags.flatten().any(|caps| {
                caps.get(1)
                    .is_some_and(|m| m.as_str().trim().to_lowercase() == lang)
            });
            if !matched {
                kept.push("");
                continue;
            }
        }

        kept.push(code);
    }

    kept.join("\n")
}

/// Every distinct `[:lang-code:]` tag declared in `source`, sorted.
///
/// Tags only ever appear inside comments (spec 2.7.1), so this looks
/// only after the `#` on each line - the same place the language filter
/// does - rather than scanning the whole text, which would also pick up
/// a bracket expression that happens to look like a tag inside a
/// pattern.
///
/// ```
/// let source = "1 one\n1 uno # [:es:] [:es-MX:]\n";
///
/// assert_eq!(soros::language_tags(source), ["es", "es-MX"]);
/// ```
#[must_use]
pub fn language_tags(source: &str) -> Vec<String> {
    let hidden = crate::escape::hide_escapes(source);
    let mut tags = BTreeSet::new();

    for line in hidden.split('\n') {
        let Some((_, comment)) = line.split_once('#') else {
            continue;
        };

        for caps in LANG_TAG_RE.captures_iter(comment).flatten() {
            if let Some(tag) = caps.get(1) {
                let tag = tag.as_str().trim();
                if !tag.is_empty() {
                    tags.insert(tag.to_owned());
                }
            }
        }
    }

    tags.into_iter().collect()
}

/// Splits comment-free source into trimmed statements, separated by `;`
/// or a newline (spec 2.1), dropping empty ones.
pub(crate) fn split_statements(code_only: &str) -> Vec<String> {
    code_only
        .split([';', '\n'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

/// Splits one statement into its regex and value parts (spec 2.2),
/// honoring optional ASCII-quote wrapping of either part (spec 2.3.3).
///
/// The statement is expected to still carry hidden escape placeholders
/// (see [`crate::escape::hide_escapes`]), which is what makes any
/// remaining `"` an unambiguous quote delimiter rather than literal
/// text.
pub(crate) fn split_pattern_value(stmt: &str) -> (String, String) {
    let (pattern, rest) = match stmt.strip_prefix('"') {
        Some(quoted) => match quoted.find('"') {
            Some(end) => (&quoted[..end], &quoted[end + 1..]),
            None => (quoted, ""),
        },
        None => match stmt.find([' ', '\t']) {
            Some(i) => (&stmt[..i], &stmt[i..]),
            None => (stmt, ""),
        },
    };

    let rest = rest.trim_start_matches([' ', '\t']);
    let value = match (rest.strip_prefix('"'), rest.strip_suffix('"')) {
        (Some(_), Some(_)) if rest.len() >= 2 => &rest[1..rest.len() - 1],
        _ => rest,
    };

    (pattern.to_owned(), value.to_owned())
}

/// Returns the prefix declared by a `== prefix ==` macro statement, or
/// `None` if `stmt` is not one.
pub(crate) fn macro_prefix(stmt: &str) -> Option<String> {
    MACRO_DIRECTIVE_RE
        .captures(stmt)
        .ok()
        .flatten()
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_owned()))
}
