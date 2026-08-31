//! The command-line front end, exercised as a subprocess.
//!
//! `CARGO_BIN_EXE_numbertext` is set by cargo to the binary built for
//! this test run, so these always test the current code rather than
//! whatever happens to be installed.

use std::process::{Command, Output};

fn numbertext(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_numbertext"))
        .args(args)
        .output()
        .expect("failed to run the numbertext binary")
}

fn stdout_of(args: &[&str]) -> String {
    let output = numbertext(args);

    assert!(
        output.status.success(),
        "{args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout)
        .expect("output must be UTF-8")
        .trim_end()
        .to_owned()
}

#[test]
fn each_action_produces_its_conversion() {
    assert_eq!(
        stdout_of(&["--cardinal", "12345"]),
        "twelve thousand three hundred forty-five"
    );
    assert_eq!(stdout_of(&["--ordinal", "21"]), "twenty-first");
    assert_eq!(stdout_of(&["--ordinal-number", "3"]), "3rd");
    assert_eq!(stdout_of(&["--year", "1999"]), "nineteen ninety-nine");
    assert_eq!(
        stdout_of(&["--currency", "USD", "2.5"]),
        "two U.S. dollars and fifty cents"
    );
    assert_eq!(
        stdout_of(&["--money", "USD", "2.5"]),
        "two and 50/100 U.S. dollars"
    );
}

#[test]
fn lang_selects_the_locale_and_accepts_a_regional_code() {
    assert_eq!(
        stdout_of(&["--lang", "pt", "--cardinal", "16"]),
        "dezasseis"
    );
    assert_eq!(
        stdout_of(&["--lang", "pt-BR", "--cardinal", "16"]),
        "dezesseis"
    );
    assert_eq!(
        stdout_of(&["--lang", "en-GB", "--cardinal", "101"]),
        "one hundred and one"
    );
}

/// A negative number is read as the flag's value, not mistaken for
/// another flag - the case a hand-rolled parser is most likely to get
/// wrong.
#[test]
fn a_negative_number_is_not_read_as_a_flag() {
    assert_eq!(
        stdout_of(&["--lang", "pt", "--cardinal", "-5"]),
        "menos cinco"
    );
}

#[test]
fn convert_reaches_a_locale_specific_section() {
    assert_eq!(
        stdout_of(&["--lang", "pt", "--convert", "feminine", "2"]),
        "duas"
    );
}

/// `--list` is what a caller - and the shell completions - read to learn
/// what `--lang` accepts, so it reports variants alongside the languages
/// that have a file. The library keeps the two apart; the CLI merges them
/// because "what can I pass" is the only question being asked here.
#[test]
fn list_reports_languages_and_their_variants() {
    let listed: Vec<String> = stdout_of(&["--list"])
        .lines()
        .map(ToOwned::to_owned)
        .collect();

    let mut expected = numbertext::languages();
    expected.extend(numbertext::language_variants());
    expected.sort();

    assert_eq!(listed, expected);

    for want in ["en", "pt", "pt-BR", "en-GB", "nn"] {
        assert!(
            listed.contains(&want.to_owned()),
            "--list is missing {want}"
        );
    }
}

/// Every code `--list` prints has to be one the CLI itself accepts.
/// Advertising a code that fails would be worse than not listing it,
/// because shell completion reads exactly this list.
#[test]
fn every_listed_code_is_usable() {
    for code in stdout_of(&["--list"]).lines() {
        let output = numbertext(&["--lang", code, "--cardinal", "1"]);

        assert!(
            output.status.success(),
            "--list advertises {code}, but --lang {code} fails: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn help_section_prints_the_locales_own_summary() {
    assert!(!stdout_of(&["--help-section"]).is_empty());
}

#[test]
fn help_exits_successfully() {
    assert!(stdout_of(&["--help"]).contains("usage: numbertext"));
}

#[test]
fn misuse_fails_with_a_message_on_stderr() {
    for args in [
        vec!["--lang", "zz-not-a-lang", "--cardinal", "1"],
        vec![],
        vec!["--cardinal"],
        vec!["--nonsense"],
    ] {
        let output = numbertext(&args);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(!output.status.success(), "{args:?} should have failed");
        assert!(
            stderr.contains("numbertext:"),
            "{args:?} printed no diagnostic"
        );
    }
}
