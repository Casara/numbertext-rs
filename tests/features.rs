//! Selective language embedding.
//!
//! Which locales a build contains is decided by Cargo features (see
//! "Selecting which languages get embedded" in README.md), so proving it
//! works takes more than one build. `make test-features` runs this file
//! under `--no-default-features --features lang-en,lang-pt`; the default
//! build compiles it too, and each case is gated to the configuration it
//! actually describes.

/// The default build embeds everything, so a caller who has expressed no
/// preference is never surprised by a missing language.
#[cfg(feature = "all-languages")]
#[test]
fn the_default_build_embeds_every_language() {
    let langs = numbertext::languages();

    assert_eq!(langs.len(), 52);
    for want in ["en", "pt", "de", "hu_Hung", "Roman"] {
        assert!(langs.contains(&want.to_owned()), "missing {want}");
    }
}

/// Opting in embeds exactly what was asked for - and nothing else.
///
/// The second half matters as much as the first: the failure mode of
/// getting this wrong is not a build error but a runtime one, so the
/// test has to pin that an unselected language is genuinely absent
/// rather than quietly still there.
#[cfg(all(
    feature = "lang-en",
    feature = "lang-pt",
    not(feature = "all-languages")
))]
#[test]
fn opting_in_embeds_exactly_the_selected_languages() {
    assert_eq!(
        numbertext::languages(),
        vec!["en".to_owned(), "pt".to_owned()]
    );

    // The languages kept behave exactly as they do in a full build.
    assert_eq!(
        numbertext::cardinal("en", "101").unwrap(),
        "one hundred one"
    );
    assert_eq!(numbertext::cardinal("pt", "16").unwrap(), "dezasseis");

    // Including their regional variants, which live inside those files.
    assert_eq!(
        numbertext::cardinal("en-GB", "101").unwrap(),
        "one hundred and one"
    );
    assert_eq!(numbertext::cardinal("pt-BR", "16").unwrap(), "dezesseis");

    // An unselected language is absent, not silently substituted.
    assert!(matches!(
        numbertext::cardinal("de", "42"),
        Err(numbertext::Error::UnknownLanguage { .. })
    ));
}

/// Embedding nothing is a legitimate configuration: it is what a program
/// supplying every locale through `register_locale` wants.
#[cfg(not(any(feature = "all-languages", feature = "lang-en")))]
#[test]
fn embedding_no_languages_still_supports_runtime_registration() {
    assert!(numbertext::languages().is_empty());

    numbertext::register_locale("mini", "1 uno\n2 dos\n").unwrap();

    assert_eq!(numbertext::cardinal("mini", "2").unwrap(), "dos");
    assert_eq!(numbertext::languages(), vec!["mini".to_owned()]);
}
