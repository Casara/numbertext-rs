//! Every bundled locale, exercised end to end.
//!
//! This does not assert exact wording - that would need a native speaker
//! per language. It exists to catch what would break a locale outright:
//! an engine change that makes a real rule file stop compiling, or a
//! data sync that introduces a malformed rule.

use numbertext::Locale;

/// A spread of inputs covering the shapes locale files branch on: zero,
/// single digits, teens, round hundreds, four-digit years, negatives,
/// and a decimal.
const NUMBERS: &[&str] = &["0", "1", "15", "42", "100", "1999", "-7", "3.14"];

/// No bundled locale may contain a rule this crate cannot compile.
///
/// This is the check that turns a silent degradation into a failure. A
/// skipped rule never matches, so a locale carrying one keeps working
/// for most input and quietly produces the wrong wording for the rest -
/// exactly the kind of regression a data sync can introduce without
/// anyone noticing.
#[test]
fn every_bundled_locale_compiles_with_no_skipped_rules() {
    let mut offenders = Vec::new();

    for lang in numbertext::languages() {
        let locale = Locale::new(&lang).expect("a listed language must resolve");

        for rule in locale.skipped_rules() {
            offenders.push(format!("{lang}: {rule}"));
        }
    }

    assert!(
        offenders.is_empty(),
        "{} rule(s) failed to compile:\n{}",
        offenders.len(),
        offenders.join("\n")
    );
}

/// Every locale answers every entry point without panicking.
#[test]
fn every_bundled_locale_converts() {
    for lang in numbertext::languages() {
        let locale = Locale::new(&lang).expect("a listed language must resolve");

        for n in NUMBERS {
            let _ = locale.cardinal(n);
        }

        let _ = locale.ordinal("3");
        let _ = locale.ordinal_number("3");
        let _ = locale.year("2001");
        let _ = locale.currency("USD", "2.5");
        let _ = locale.money("USD", "2.5");
        let _ = locale.help();
    }
}

/// Cardinal conversion produces *something* for a plain integer in every
/// locale. Weaker than checking the wording, but strong enough to catch
/// a locale whose entry rule stopped matching entirely.
#[test]
fn every_bundled_locale_spells_out_a_plain_integer() {
    for lang in numbertext::languages() {
        let locale = Locale::new(&lang).expect("a listed language must resolve");

        assert!(
            !locale.cardinal("42").is_empty(),
            "{lang}: cardinal(42) produced nothing"
        );
    }
}

/// The bundled set is the upstream set: 52 rule files. A locale silently
/// dropped by a bad sync or a build.rs change shows up here.
#[test]
fn the_expected_number_of_locales_is_bundled() {
    assert_eq!(
        numbertext::languages().len(),
        52,
        "expected the 52 upstream locales; update this count deliberately \
         after a sync that adds or removes one"
    );
}
