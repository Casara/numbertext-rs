//! Public API behavior, against the bundled locale files.
//!
//! Every expected value here was produced by running the code and
//! checking it against the rule file that generated it - `.sor` files
//! encode real per-language stylistic choices that are easy to get wrong
//! by intuition. Where a value looks surprising, the comment says which
//! rule makes it so.

use numbertext::{Error, Locale};

#[test]
fn cardinal_english() {
    let en = Locale::new("en").unwrap();

    // Plain "en" (no country variant) deliberately omits "and" before
    // the tens/units part of a hundred - en.sor's en-AU/GB/IE/NZ-tagged
    // rule adds it, the untagged default does not. And "0.x" reads as
    // "point x" with no leading "zero", from en.sor's dedicated
    // "0[.,] point" rule.
    for (input, want) in [
        ("0", "zero"),
        ("1", "one"),
        ("9", "nine"),
        ("10", "ten"),
        ("11", "eleven"),
        ("19", "nineteen"),
        ("20", "twenty"),
        ("21", "twenty-one"),
        ("99", "ninety-nine"),
        ("100", "one hundred"),
        ("101", "one hundred one"),
        ("999", "nine hundred ninety-nine"),
        ("1000", "one thousand"),
        // The separator function inserts "and" before a sub-100
        // remainder, which is why 1001 has it and 101 does not.
        ("1001", "one thousand and one"),
        ("12345", "twelve thousand three hundred forty-five"),
        ("1000000", "one million"),
        ("-5", "negative five"),
        ("-123", "negative one hundred twenty-three"),
        ("0.5", "point five"),
        ("3.14", "three point one four"),
        ("1000000000", "one billion"),
        ("1000000000000", "one trillion"),
    ] {
        assert_eq!(en.cardinal(input), want, "cardinal({input})");
    }
}

#[test]
fn cardinal_from_an_integer() {
    let en = Locale::new("en").unwrap();

    assert_eq!(en.cardinal_i64(42), "forty-two");
    assert_eq!(en.cardinal_i64(-42), "negative forty-two");
    assert_eq!(en.cardinal_i64(0), "zero");
}

#[test]
fn ordinal_english() {
    let en = Locale::new("en").unwrap();

    for (input, want) in [
        ("1", "first"),
        ("2", "second"),
        ("3", "third"),
        ("5", "fifth"),
        ("8", "eighth"),
        ("9", "ninth"),
        ("12", "twelfth"),
        ("20", "twentieth"),
        ("21", "twenty-first"),
        ("100", "one hundredth"),
    ] {
        assert_eq!(en.ordinal(input), want, "ordinal({input})");
    }
}

#[test]
fn ordinal_number_english() {
    let en = Locale::new("en").unwrap();

    for (input, want) in [
        ("1", "1st"),
        ("2", "2nd"),
        ("3", "3rd"),
        ("4", "4th"),
        ("11", "11th"),
        ("12", "12th"),
        ("13", "13th"),
        ("21", "21st"),
        ("22", "22nd"),
        ("23", "23rd"),
    ] {
        assert_eq!(en.ordinal_number(input), want, "ordinal_number({input})");
    }
}

#[test]
fn year_english() {
    let en = Locale::new("en").unwrap();

    // en.sor reads a 19xx year as two two-digit groups, and runs
    // everything else through a "year-remove-and" pass that plain
    // `cardinal` never sees - which is why 2001 loses the "and" that
    // cardinal("1001") keeps.
    assert_eq!(en.year("1999"), "nineteen ninety-nine");
    assert_eq!(en.year("2000"), "two thousand");
    assert_eq!(en.year("2001"), "two thousand one");
    assert_eq!(en.cardinal("2001"), "two thousand and one");
}

#[test]
fn currency_english() {
    let en = Locale::new("en").unwrap();

    assert_eq!(en.currency("USD", "1"), "one U.S. dollar");
    assert_eq!(
        en.currency("USD", "2.5"),
        "two U.S. dollars and fifty cents"
    );
    assert_eq!(en.currency("EUR", "1"), "one euro");
    assert_eq!(en.currency("EUR", "2"), "two euro");
}

#[test]
fn money_english() {
    let en = Locale::new("en").unwrap();

    // The MONEYTEXT-style section mirrors check-writing wording: amount
    // and fraction first, unit name last.
    assert_eq!(en.money("USD", "2.5"), "two and 50/100 U.S. dollars");
}

/// A regional code with no `.sor` file of its own falls back to its base
/// language's file while still activating that region's
/// `[:lang-code:]`-tagged lines (spec 2.7.1) - how `en.sor` and `pt.sor`
/// carry regional wording differences inside one file.
#[test]
fn regional_variants_fall_back_to_the_base_file() {
    assert_eq!(
        Locale::new("en-GB").unwrap().cardinal("101"),
        "one hundred and one"
    );
    assert_eq!(
        Locale::new("en").unwrap().cardinal("101"),
        "one hundred one"
    );
    assert_eq!(Locale::new("pt-BR").unwrap().cardinal("16"), "dezesseis");
    assert_eq!(Locale::new("pt").unwrap().cardinal("16"), "dezasseis");

    // The requested code is preserved, not collapsed to the base file.
    assert_eq!(Locale::new("en-GB").unwrap().code(), "en-GB");
}

#[test]
fn an_unknown_base_language_is_an_error() {
    assert_eq!(
        Locale::new("xx-YY").unwrap_err(),
        Error::UnknownLanguage {
            lang: "xx-YY".to_owned()
        }
    );
    assert!(matches!(
        numbertext::cardinal("xx-not-a-lang", "1"),
        Err(Error::UnknownLanguage { .. })
    ));
}

#[test]
fn help_reports_the_locales_own_usage_text() {
    assert!(!Locale::new("en").unwrap().help().is_empty());
}

/// Locale-specific sections reached through `convert`, e.g. `pt.sor`'s
/// gender variants. These names are not standardized across languages,
/// which is why there is no `gender()` method.
#[test]
fn locale_specific_sections_via_convert() {
    let pt = Locale::new("pt").unwrap();

    assert_eq!(pt.cardinal("2"), "dois");
    assert_eq!(pt.convert("feminine", "2"), "duas");
}

/// `hu.sor`'s grammatical-case suffix lookup: the one real-world use of
/// a backreference inside a *matching* pattern across every bundled
/// locale. A "database" of case/suffix pairs is packed into one string
/// and searched with `.*:\1:(-[^-:]*).*`, where `\1` correlates the
/// requested case name (captured earlier, from the macro prefix) with
/// its entry.
///
/// The expected value comes from `hu.sor`'s own help section
/// ("-szor 15 -> 15-ször").
#[test]
fn hungarian_suffix_database_lookup() {
    assert_eq!(Locale::new("hu").unwrap().convert("-szor", "15"), "15-ször");
}

/// The `bg.sor` ordinal-thousands rule whose upstream text has a
/// bracket and paren transposed (`\2)])` for `\2))]`), patched locally
/// pending <https://github.com/Numbertext/libnumbertext/issues/139> -
/// see `data/SOURCE.md`'s "Local patches" section.
#[test]
fn bulgarian_ordinal_thousands() {
    let bg = Locale::new("bg").unwrap();

    assert_eq!(bg.ordinal("1023"), "хиляда двадесет и трети");
    assert_eq!(bg.ordinal("2023"), "две хиляди двадесет и трети");
    assert_eq!(bg.ordinal("5023"), "пет хиляди двадесет и трети");
    assert!(
        bg.skipped_rules().is_empty(),
        "the patched rule must compile"
    );
}

#[test]
fn languages_lists_bundled_locales() {
    let langs = numbertext::languages();

    for want in ["en", "pt", "de", "hu_Hung"] {
        assert!(
            langs.contains(&want.to_owned()),
            "languages() missing {want}"
        );
    }
    // Regional variants have no file of their own, so they never appear.
    assert!(!langs.contains(&"en-GB".to_owned()));
}

#[test]
fn register_locale_adds_a_language_at_runtime() {
    numbertext::register_locale("test-es-mini", "1 uno\n2 dos\n3 tres\n").unwrap();

    assert_eq!(numbertext::cardinal("test-es-mini", "2").unwrap(), "dos");
    assert!(numbertext::languages().contains(&"test-es-mini".to_owned()));
}

#[test]
fn register_locale_rejects_an_empty_code() {
    assert_eq!(
        numbertext::register_locale("", "1 uno\n").unwrap_err(),
        Error::EmptyLocaleCode
    );
}

/// Re-registering a code replaces it, and a `Locale` already resolved
/// from the previous source keeps working with what it compiled.
#[test]
fn register_locale_replaces_and_existing_handles_are_snapshots() {
    numbertext::register_locale("test-replace", "1 first-version\n").unwrap();
    let before = Locale::new("test-replace").unwrap();

    numbertext::register_locale("test-replace", "1 second-version\n").unwrap();
    let after = Locale::new("test-replace").unwrap();

    assert_eq!(before.cardinal("1"), "first-version");
    assert_eq!(after.cardinal("1"), "second-version");
}

/// The free functions are exactly `Locale::new(lang)?.method(..)`.
#[test]
fn free_functions_match_the_locale_methods() {
    let en = Locale::new("en").unwrap();

    assert_eq!(numbertext::cardinal("en", "42").unwrap(), en.cardinal("42"));
    assert_eq!(numbertext::ordinal("en", "42").unwrap(), en.ordinal("42"));
    assert_eq!(
        numbertext::ordinal_number("en", "42").unwrap(),
        en.ordinal_number("42")
    );
    assert_eq!(numbertext::year("en", "1999").unwrap(), en.year("1999"));
    assert_eq!(
        numbertext::currency("en", "USD", "2.5").unwrap(),
        en.currency("USD", "2.5")
    );
    assert_eq!(
        numbertext::money("en", "USD", "2.5").unwrap(),
        en.money("USD", "2.5")
    );
    assert_eq!(numbertext::help("en").unwrap(), en.help());
    assert_eq!(
        numbertext::convert("pt", "feminine", "2").unwrap(),
        Locale::new("pt").unwrap().convert("feminine", "2")
    );
}

/// Input no rule matches yields an empty string - the language has no
/// notion of a failed conversion, so this is not an error.
#[test]
fn unmatched_input_yields_an_empty_string() {
    assert_eq!(Locale::new("en").unwrap().cardinal("not-a-number"), "");
}

/// The public types cross thread boundaries.
///
/// This is a compile-time assertion, not a runtime one: it fails to
/// build rather than to run. Worth pinning explicitly because it is an
/// easy guarantee to lose by accident - swapping an `Arc` for an `Rc`
/// inside the registry, say - and losing it breaks every caller that
/// holds a `Locale` in shared state, with an error message pointing at
/// their code rather than at the change that caused it.
#[test]
fn public_types_are_send_and_sync() {
    const fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<Locale>();
    assert_send_sync::<Error>();
    assert_send_sync::<numbertext::SkippedRule>();
}

/// The registry is shared process-wide, so concurrent first-use of the
/// same and of different languages has to be safe - including the race
/// where two threads both find a language uncached and both compile it.
#[test]
fn concurrent_first_use_is_safe() {
    let langs = ["en", "pt", "de", "fr", "es", "it", "ru", "hu"];

    std::thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(|| {
                for lang in langs {
                    let locale = Locale::new(lang).expect("bundled language must resolve");
                    assert!(!locale.cardinal("42").is_empty(), "{lang}");
                }
                // Registering while other threads are looking things up.
                numbertext::register_locale("test-concurrent", "1 one\n").unwrap();
            });
        }
    });

    assert_eq!(numbertext::cardinal("test-concurrent", "1").unwrap(), "one");
}

/// A script variant lives in a file whose name uses `_`, but a caller
/// naturally writes the code with `-`. Upstream resolves that by
/// rewriting the separator before looking the file up, and this port has
/// to as well.
///
/// Getting it wrong is not an error but a *silently wrong answer*:
/// `hu-Hung` would fall back to `hu.sor` and return ordinary Hungarian
/// where Old Hungarian script was asked for.
#[test]
fn script_variants_resolve_with_either_separator() {
    let underscore = Locale::new("hu_Hung").unwrap();
    let dash = Locale::new("hu-Hung").unwrap();

    assert_eq!(underscore.cardinal("1"), dash.cardinal("1"));
    assert_ne!(
        dash.cardinal("1"),
        Locale::new("hu").unwrap().cardinal("1"),
        "hu-Hung must not silently fall back to plain Hungarian"
    );
}

/// Norwegian is the one language upstream special-cases by name
/// (`LANG_PATTERN_NO`): Bokmål and Nynorsk share `no.sor` and neither has
/// a file of its own, so `nb`/`nn` have to resolve to it while keeping
/// the requested code, which is what activates the tagged lines.
///
/// Without this, `no.sor`'s `[:nn:]`/`[:nn-NO:]` lines are unreachable by
/// any code at all - the wording they encode could never be produced.
#[test]
fn norwegian_written_standards_resolve_to_the_shared_file() {
    for code in ["nb", "nb-NO"] {
        assert_eq!(Locale::new(code).unwrap().cardinal("1"), "én", "{code}");
    }
    for code in ["nn", "nn-NO"] {
        assert_eq!(Locale::new(code).unwrap().cardinal("1"), "éin", "{code}");
    }

    // The bare "no" file keeps its own default, Bokmål.
    assert_eq!(Locale::new("no").unwrap().cardinal("1"), "én");
}

/// The resolution order must not rewrite a code that matches a source
/// exactly, or a runtime locale registered under a name containing `-`
/// becomes unreachable.
#[test]
fn an_exact_match_wins_over_any_rewriting() {
    numbertext::register_locale("test-exact-match", "1 uno\n").unwrap();

    assert_eq!(
        Locale::new("test-exact-match").unwrap().cardinal("1"),
        "uno"
    );
}

/// Variants are read out of the rule files, not listed in code, so a data
/// sync picks up a new one without a change here.
#[test]
fn language_variants_are_discovered_from_the_rule_files() {
    let variants = numbertext::language_variants();

    for want in [
        "pt-BR",
        "en-GB",
        "de-CH",
        "fr-BE",
        "ko-KP",
        "nn",
        "Roman-large",
    ] {
        assert!(variants.contains(&want.to_owned()), "missing {want}");
    }

    // A code with a file of its own is a language, not a variant.
    for base in numbertext::languages() {
        assert!(
            !variants.contains(&base),
            "{base} has a file; it is a language"
        );
    }
}

/// Every variant this crate advertises has to actually resolve and
/// produce output. Listing a code that errors would be worse than not
/// listing it - shell completion reads this list.
#[test]
fn every_advertised_variant_works() {
    for variant in numbertext::language_variants() {
        let locale = Locale::new(&variant)
            .unwrap_or_else(|err| panic!("advertised variant {variant} does not resolve: {err}"));

        assert!(
            !locale.cardinal("1").is_empty(),
            "advertised variant {variant} produced nothing"
        );
    }
}

/// A variant has to change something, or it would not be worth
/// advertising as distinct from its base.
#[test]
fn a_variant_differs_from_its_base() {
    for (variant, base, n) in [
        ("pt-BR", "pt", "16"),
        ("en-GB", "en", "101"),
        ("nn", "no", "1"),
    ] {
        assert_ne!(
            Locale::new(variant).unwrap().cardinal(n),
            Locale::new(base).unwrap().cardinal(n),
            "{variant} should differ from {base}"
        );
    }
}

/// `lg.sor` has `1(\d{6}) kakadde [ka$(:1) ne $1]`, where the bracket
/// holds two calls and the second is empty for a round million. Treating
/// a `[...]` block as conditional on *every* call inside it - rather
/// than on its first - silently truncated these, and nothing else in the
/// bundled data distinguishes the two rules.
///
/// Expected values come from upstream's own `spellout`.
#[test]
fn luganda_keeps_a_block_whose_later_call_is_empty() {
    let lg = Locale::new("lg").unwrap();

    // The trailing space is upstream's too, byte for byte: the block is
    // kept and the empty second call simply contributes nothing after
    // the space that precedes it. Not this port's to trim.
    assert_eq!(lg.cardinal("1000000"), "kakadde kamu ne ");
    assert_eq!(lg.cardinal("10000"), "mutwalo gumu mu ");
    // The ending is not spurious: a non-round value keeps it and adds to it.
    assert_eq!(lg.cardinal("1000001"), "kakadde kamu ne emu");
}
