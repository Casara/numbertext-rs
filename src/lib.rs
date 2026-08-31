//! Spell out numbers as words - cardinals, ordinals, years, currency
//! amounts, and gender/case variants - in every language shipped by
//! [Numbertext.org](https://numbertext.github.io/).
//!
//! Each language is described by a small rule file (a `.sor` program in
//! the [Soros] rewriting language) rather than by hand-written Rust, so
//! adding a language - or a regional or gender variant of one - is a
//! data change, never a code change. See [`register_locale`] to add one
//! at runtime.
//!
//! # Quick start
//!
//! ```
//! # fn main() -> Result<(), numbertext::Error> {
//! let en = numbertext::Locale::new("en")?;
//!
//! assert_eq!(en.cardinal("12345"), "twelve thousand three hundred forty-five");
//! assert_eq!(en.ordinal("21"), "twenty-first");
//! assert_eq!(en.year("1999"), "nineteen ninety-nine");
//! assert_eq!(en.currency("USD", "2.50"), "two U.S. dollars and fifty cents");
//! # Ok(())
//! # }
//! ```
//!
//! # Two ways in
//!
//! [`Locale`] resolves and compiles a language once, then converts
//! without any further fallible step - a conversion itself cannot fail.
//! Prefer it whenever you convert more than one number for the same
//! language.
//!
//! The free functions ([`cardinal`], [`ordinal`], [`year`], ...) are
//! one-shot conveniences that take the language code per call and fold
//! the lookup into the return type:
//!
//! ```
//! assert_eq!(numbertext::cardinal("de", "42")?, "zweiundvierzig");
//! # Ok::<(), numbertext::Error>(())
//! ```
//!
//! Both go through the same process-wide cache, so neither recompiles a
//! language that has already been used.
//!
//! # Numbers are strings
//!
//! Every numeric argument is a `&str` (`"123"`, `"-5"`, `"3.14"`) rather
//! than an integer or float. The engine is text-based, and a `.sor` file
//! can spell out numbers far past any fixed-width type, so taking them
//! as text keeps arbitrary precision exact instead of quietly rounding
//! it away. [`Locale::cardinal_i64`] covers the common integer case.
//!
//! [Soros]: soros

#![doc(html_root_url = "https://docs.rs/numbertext")]

// Compiles and runs every Rust example in README.md as a doctest,
// without putting the README into the rendered API docs (where it would
// duplicate the module documentation above).
//
// This is what makes the README's examples verified rather than merely
// plausible: a signature change that invalidates one of them fails
// `cargo test --doc`, in the same run as everything else. A port to a
// language without doctests has to write and maintain a script to get
// this.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeExamples;

// The translated README's examples are held to the same standard, for
// the same reason: a code sample nobody runs is a code sample that
// drifts, and the pt-BR side drifts more easily because it gets edited
// less often.
#[cfg(doctest)]
#[doc = include_str!("../README.pt-BR.md")]
struct ExemplosDoLeiame;

mod error;
mod registry;

use std::sync::Arc;

use soros::Program;

pub use error::Error;
pub use soros::SkippedRule;

/// A language, resolved and compiled once.
///
/// Getting one can fail (the code may not exist); using one cannot.
/// Every conversion method returns a [`String`] directly, because the
/// underlying language has no notion of a failed conversion: input no
/// rule matches produces an empty string.
///
/// A `Locale` is an immutable snapshot. If
/// [`register_locale`] later replaces the code this handle was built
/// from, the handle keeps converting with the program it already
/// compiled; resolve a new one to pick up the replacement.
///
/// ```
/// # fn main() -> Result<(), numbertext::Error> {
/// let pt = numbertext::Locale::new("pt-BR")?;
///
/// assert_eq!(pt.cardinal("16"), "dezesseis");
/// assert_eq!(pt.cardinal("-5"), "menos cinco");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Locale {
    code: String,
    program: Arc<Program>,
}

impl Locale {
    /// Resolves `lang` and compiles its rule file, reusing the
    /// process-wide cache when it has been compiled before.
    ///
    /// `lang` is a `.sor` file's base name (`"en"`, `"pt"`, the
    /// Hungarian long-scale variant `"hu_Hung"`) or a regional variant
    /// built on one (`"en-GB"`, `"pt-BR"`) - see
    /// [regional variants](#regional-variants).
    ///
    /// # Errors
    ///
    /// [`Error::UnknownLanguage`] if neither `lang` nor its base
    /// language has a bundled or registered rule file.
    ///
    /// # Regional variants
    ///
    /// A regional code has no rule file of its own. `en.sor` carries the
    /// base language's rules plus lines tagged `[:en-GB:]`, `[:en-AU:]`
    /// and so on; passing the full code falls back to the base file and
    /// activates that region's tagged lines:
    ///
    /// ```
    /// # fn main() -> Result<(), numbertext::Error> {
    /// let gb = numbertext::Locale::new("en-GB")?;
    /// let us = numbertext::Locale::new("en")?;
    ///
    /// assert_eq!(gb.cardinal("101"), "one hundred and one");
    /// assert_eq!(us.cardinal("101"), "one hundred one");
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(lang: &str) -> Result<Self, Error> {
        Ok(Self {
            code: lang.to_owned(),
            program: registry::program(lang)?,
        })
    }

    /// The language code this locale was resolved for, exactly as it was
    /// requested - `"en-GB"` stays `"en-GB"`, not the `"en"` file it
    /// read its rules from.
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// The primitive every other conversion is built on.
    ///
    /// It assembles one input string and runs the language's program
    /// against it:
    ///
    /// - `prefix` empty: the input is just `arg` - what [`Locale::cardinal`] does.
    /// - `arg` empty: the input is just `prefix` - what [`Locale::help`] does.
    /// - otherwise: the input is `"{prefix} {arg}"` - what
    ///   [`Locale::ordinal`], [`Locale::year`], [`Locale::currency`] and
    ///   [`Locale::money`] do.
    ///
    /// This mirrors how a `.sor` file's own `help` section documents its
    /// sections, e.g. `$(ordinal 1)` or `$(year 1999)`. Reach for it
    /// directly when you need a section none of the named methods
    /// cover: a language's gender or grammatical-case variants, whose
    /// names are **not** standardized across languages.
    ///
    ///
    /// ```
    /// # fn main() -> Result<(), numbertext::Error> {
    /// let pt = numbertext::Locale::new("pt")?;
    ///
    /// assert_eq!(pt.cardinal("2"), "dois");
    /// assert_eq!(pt.convert("feminine", "2"), "duas");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Check [`Locale::help`], or the language's `.sor` file, for the
    /// section names it actually defines.
    #[must_use]
    pub fn convert(&self, prefix: &str, arg: &str) -> String {
        let input = match (prefix.is_empty(), arg.is_empty()) {
            (true, _) => arg.to_owned(),
            (false, true) => prefix.to_owned(),
            (false, false) => format!("{prefix} {arg}"),
        };

        self.program.run(&input)
    }

    /// Spells out `n` as a cardinal number.
    ///
    /// ```
    /// # let en = numbertext::Locale::new("en").unwrap();
    /// assert_eq!(en.cardinal("123"), "one hundred twenty-three");
    /// assert_eq!(en.cardinal("3.14"), "three point one four");
    /// ```
    #[must_use]
    pub fn cardinal(&self, n: &str) -> String {
        self.convert("", n)
    }

    /// Spells out `n` as a cardinal number, for the common integer case.
    ///
    /// ```
    /// # let en = numbertext::Locale::new("en").unwrap();
    /// assert_eq!(en.cardinal_i64(-42), "negative forty-two");
    /// ```
    #[must_use]
    pub fn cardinal_i64(&self, n: i64) -> String {
        self.cardinal(&n.to_string())
    }

    /// Spells out `n` as an ordinal number.
    ///
    /// ```
    /// # let en = numbertext::Locale::new("en").unwrap();
    /// assert_eq!(en.ordinal("123"), "one hundred twenty-third");
    /// ```
    #[must_use]
    pub fn ordinal(&self, n: &str) -> String {
        self.convert("ordinal", n)
    }

    /// Formats `n` as an abbreviated ordinal.
    ///
    /// ```
    /// # let en = numbertext::Locale::new("en").unwrap();
    /// assert_eq!(en.ordinal_number("123"), "123rd");
    /// ```
    #[must_use]
    pub fn ordinal_number(&self, n: &str) -> String {
        self.convert("ordinal-number", n)
    }

    /// Spells out `n` as a calendar year.
    ///
    /// Not a synonym for [`Locale::cardinal`]: a language may read a
    /// year quite differently, and English does.
    ///
    /// ```
    /// # let en = numbertext::Locale::new("en").unwrap();
    /// assert_eq!(en.year("1999"), "nineteen ninety-nine");
    /// assert_eq!(en.cardinal("1999"), "one thousand nine hundred ninety-nine");
    /// ```
    #[must_use]
    pub fn year(&self, n: &str) -> String {
        self.convert("year", n)
    }

    /// Spells out `amount` of the ISO 4217 currency `iso_code`, in the
    /// `NUMBERTEXT()`-compatible form.
    ///
    /// See [`Locale::money`] for the `MONEYTEXT()`-compatible variant,
    /// which some locales format differently. They are two defined
    /// renderings of the same amount, not interchangeable spellings.
    ///
    /// ```
    /// # let en = numbertext::Locale::new("en").unwrap();
    /// assert_eq!(en.currency("USD", "2.5"), "two U.S. dollars and fifty cents");
    /// ```
    #[must_use]
    pub fn currency(&self, iso_code: &str, amount: &str) -> String {
        self.convert("", &format!("{iso_code} {amount}"))
    }

    /// Spells out `amount` of `iso_code` using the locale's `money`
    /// section - the `MONEYTEXT()`-compatible form, which mirrors
    /// check-writing wording in several languages.
    ///
    /// ```
    /// # let en = numbertext::Locale::new("en").unwrap();
    /// assert_eq!(en.money("USD", "2.5"), "two and 50/100 U.S. dollars");
    /// ```
    #[must_use]
    pub fn money(&self, iso_code: &str, amount: &str) -> String {
        self.convert("money", &format!("{iso_code} {amount}"))
    }

    /// The locale's self-documenting `help` section, if it defines one:
    /// a short usage summary naming the sections available for that
    /// language.
    ///
    /// Meant for interactive exploration, not for programmatic parsing -
    /// the text is upstream prose and can change with a data sync.
    /// Returns an empty string for a locale that defines no `help`
    /// section.
    #[must_use]
    pub fn help(&self) -> String {
        self.convert("help", "")
    }

    /// The rule-file lines that could not be compiled, if any.
    ///
    /// Empty for every bundled locale - `tests/smoke.rs` asserts exactly
    /// that, which is how a data sync that introduces a malformed rule
    /// gets caught. Worth checking for a locale you supplied yourself
    /// via [`register_locale`]: a rule listed here simply never matches,
    /// so a typo shows up as wording that silently falls through to
    /// another rule rather than as an error.
    ///
    /// ```
    /// # fn main() -> Result<(), numbertext::Error> {
    /// numbertext::register_locale("xx-broken", "1 one\n( unclosed-pattern\n")?;
    /// let broken = numbertext::Locale::new("xx-broken")?;
    ///
    /// assert_eq!(broken.cardinal("1"), "one");
    /// assert_eq!(broken.skipped_rules().len(), 1);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn skipped_rules(&self) -> &[SkippedRule] {
        self.program.skipped()
    }
}

/// The language codes currently available: the locales bundled into this
/// build plus anything added with [`register_locale`].
///
/// Each code is a `.sor` file's base name. Regional variants are
/// deliberately absent - `"en-GB"` has no file of its own - so their
/// absence here does **not** mean [`Locale::new`] will reject them.
///
/// ```
/// let langs = numbertext::languages();
///
/// assert!(langs.iter().any(|l| l == "en"));
/// assert!(!langs.iter().any(|l| l == "en-GB")); // still works with Locale::new
/// ```
#[must_use]
pub fn languages() -> Vec<String> {
    registry::languages()
}

/// The variant codes the available locales declare, sorted.
///
/// A variant has no rule file of its own: it is a set of
/// `[:lang-code:]`-tagged lines inside a base language's file, activated
/// by passing the full code to [`Locale::new`]. `pt.sor` carries
/// `pt-BR`, `en.sor` carries `en-GB` and its siblings, and `Roman.sor`
/// carries `Roman-large`, a notation variant rather than a regional one.
///
/// This complements [`languages`], which lists only codes backed by a
/// file. Neither list is exhaustive of what [`Locale::new`] *accepts*:
/// any `"<known-base>-<anything>"` resolves to the base file, it simply
/// activates no tagged lines. What this returns is the set of codes that
/// actually change the output.
///
/// ```
/// let variants = numbertext::language_variants();
///
/// assert!(variants.iter().any(|v| v == "pt-BR"));
/// assert!(variants.iter().any(|v| v == "en-GB"));
/// // Codes that have a file of their own belong to `languages()`.
/// assert!(!variants.iter().any(|v| v == "pt"));
/// ```
#[must_use]
pub fn language_variants() -> Vec<String> {
    registry::language_variants()
}

/// Adds a language at runtime from raw Soros source - the contents of a
/// `.sor` file - without a code change or a rebuild.
///
/// This is the intended way to use a language Numbertext.org does not
/// ship yet. `code` becomes what you pass to [`Locale::new`] and the
/// free functions. Registering a code that already exists replaces it.
///
/// ```
/// # fn main() -> Result<(), numbertext::Error> {
/// numbertext::register_locale("es-mini", "1 uno\n2 dos\n3 tres\n")?;
///
/// assert_eq!(numbertext::cardinal("es-mini", "2")?, "dos");
/// # Ok(())
/// # }
/// ```
///
/// Compilation is deferred to first use and cannot fail as a whole; a
/// malformed individual rule is skipped rather than rejected. Check
/// [`Locale::skipped_rules`] if you want to be sure your source compiled
/// cleanly.
///
/// # Errors
///
/// [`Error::EmptyLocaleCode`] if `code` is empty.
///
/// # Trust
///
/// Treat `sor_source` as configuration you control, not as untrusted
/// input: it compiles to regular expressions and drives a recursive
/// evaluator. See this crate's `SECURITY.md` for the bounds that apply.
pub fn register_locale(code: &str, sor_source: &str) -> Result<(), Error> {
    if code.is_empty() {
        return Err(Error::EmptyLocaleCode);
    }

    registry::register(code, sor_source.to_owned());

    Ok(())
}

/// Generates the one-shot free-function form of a [`Locale`] method.
///
/// These exist so a single conversion doesn't need a handle, and each
/// one is exactly `Locale::new(lang)?.<method>(..)`. Generating them
/// keeps that promise literally true - a method and its free function
/// cannot drift apart - and keeps the doc examples the only prose worth
/// reading twice.
macro_rules! one_shot {
    ($(#[$meta:meta])* $name:ident($($arg:ident),*)) => {
        $(#[$meta])*
        ///
        /// # Errors
        ///
        /// [`Error::UnknownLanguage`] if `lang` cannot be resolved - see
        /// [`Locale::new`].
        pub fn $name(lang: &str $(, $arg: &str)*) -> Result<String, Error> {
            Ok(Locale::new(lang)?.$name($($arg),*))
        }
    };
}

one_shot! {
    /// Spells out `n` as a cardinal number.
    ///
    /// ```
    /// assert_eq!(numbertext::cardinal("en", "123")?, "one hundred twenty-three");
    /// # Ok::<(), numbertext::Error>(())
    /// ```
    cardinal(n)
}

one_shot! {
    /// Spells out `n` as an ordinal number.
    ///
    /// ```
    /// assert_eq!(numbertext::ordinal("en", "123")?, "one hundred twenty-third");
    /// # Ok::<(), numbertext::Error>(())
    /// ```
    ordinal(n)
}

one_shot! {
    /// Formats `n` as an abbreviated ordinal.
    ///
    /// ```
    /// assert_eq!(numbertext::ordinal_number("en", "123")?, "123rd");
    /// # Ok::<(), numbertext::Error>(())
    /// ```
    ordinal_number(n)
}

one_shot! {
    /// Spells out `n` as a calendar year.
    ///
    /// ```
    /// assert_eq!(numbertext::year("en", "1999")?, "nineteen ninety-nine");
    /// # Ok::<(), numbertext::Error>(())
    /// ```
    year(n)
}

one_shot! {
    /// Spells out `amount` of the ISO 4217 currency `iso_code`
    /// (`NUMBERTEXT()`-compatible - see [`Locale::currency`]).
    ///
    /// ```
    /// assert_eq!(numbertext::currency("en", "EUR", "1")?, "one euro");
    /// # Ok::<(), numbertext::Error>(())
    /// ```
    currency(iso_code, amount)
}

one_shot! {
    /// Spells out `amount` of `iso_code` in the `MONEYTEXT()`-compatible
    /// form (see [`Locale::money`]).
    ///
    /// ```
    /// assert_eq!(numbertext::money("en", "USD", "2.5")?, "two and 50/100 U.S. dollars");
    /// # Ok::<(), numbertext::Error>(())
    /// ```
    money(iso_code, amount)
}

one_shot! {
    /// The locale's self-documenting `help` section (see
    /// [`Locale::help`]).
    ///
    /// ```
    /// assert!(!numbertext::help("en")?.is_empty());
    /// # Ok::<(), numbertext::Error>(())
    /// ```
    help()
}

/// Runs `lang`'s program against an input built from `prefix` and `arg`
/// (see [`Locale::convert`]).
///
/// ```
/// assert_eq!(numbertext::convert("pt", "feminine", "2")?, "duas");
/// # Ok::<(), numbertext::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::UnknownLanguage`] if `lang` cannot be resolved.
pub fn convert(lang: &str, prefix: &str, arg: &str) -> Result<String, Error> {
    Ok(Locale::new(lang)?.convert(prefix, arg))
}
