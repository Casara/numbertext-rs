//! The error type this crate returns.

use std::fmt;

/// Something that went wrong resolving or registering a locale.
///
/// Note what is *not* here: converting a number never fails. Once a
/// [`Locale`](crate::Locale) has been resolved, every conversion method
/// returns a plain [`String`], because the Soros language has no notion
/// of a failed conversion - a number no rule matches yields an empty
/// string, not an error. Errors belong to locale lookup, which is the
/// only step that can genuinely not be satisfied.
///
/// Match on the variants rather than on the [`Display`](fmt::Display)
/// text: the wording is not part of the public API.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The language code - and, for a regional variant such as
    /// `"en-GB"`, its base language too - has neither a bundled source
    /// nor one added via [`register_locale`](crate::register_locale).
    UnknownLanguage {
        /// The code that was requested.
        lang: String,
    },

    /// [`register_locale`](crate::register_locale) was called with an
    /// empty code, which no caller could ever look up again.
    EmptyLocaleCode,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownLanguage { lang } => write!(
                f,
                "unknown language {lang:?} (see `languages()` for the supported base codes; \
                 a regional variant like {lang:?} has to build on one of them, \
                 the way \"en-GB\" builds on \"en\")"
            ),
            Self::EmptyLocaleCode => f.write_str("locale code must not be empty"),
        }
    }
}

impl std::error::Error for Error {}
