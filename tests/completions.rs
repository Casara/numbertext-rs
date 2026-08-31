//! The shell completions offer exactly the flags the CLI accepts.
//!
//! Completion scripts are the one part of this project that duplicates a
//! fact instead of deriving it: the flag list lives both in
//! `src/bin/numbertext.rs` and in three hand-written scripts. That
//! duplication is deliberate - generating the scripts would mean adopting
//! a CLI-parsing crate, which would land in the dependency tree of every
//! library consumer (see the note at the top of the binary).
//!
//! What is not acceptable is letting the copies drift silently: a flag
//! added to the CLI and forgotten in the scripts produces a tool whose
//! completion quietly stops covering it. This test is what makes that a
//! build failure instead.
//!
//! Language codes are deliberately *not* duplicated - every script asks
//! `numbertext --list` at completion time, so they track the languages a
//! given build actually embedded.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

/// The repository root, which is this package's manifest directory.
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every long flag the CLI advertises in its own usage text.
fn flags_from_help() -> BTreeSet<String> {
    let output = Command::new(env!("CARGO_BIN_EXE_numbertext"))
        .arg("--help")
        .output()
        .expect("failed to run the numbertext binary");

    let help = String::from_utf8(output.stdout).expect("--help output must be UTF-8");
    let flags = long_flags(&help);

    assert!(
        !flags.is_empty(),
        "no flags found in --help; the extraction below is wrong, not the scripts"
    );

    flags
}

/// Extracts `--long-flag` tokens from arbitrary text.
///
/// A bare `--` (bash's `compgen -W ... -- "$cur"`) is not a flag and is
/// skipped, since a letter is required after the dashes.
fn long_flags(text: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();

    for token in text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-')) {
        let Some(name) = token.strip_prefix("--") else {
            continue;
        };
        if name.starts_with(|c: char| c.is_ascii_lowercase()) {
            found.insert(format!("--{}", name.trim_end_matches('-')));
        }
    }

    found
}

fn read(relative: &str) -> String {
    let path: PathBuf = root().join(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()))
}

fn assert_covers(script: &str, found: &BTreeSet<String>, expected: &BTreeSet<String>) {
    let missing: Vec<_> = expected.difference(found).collect();
    let extra: Vec<_> = found.difference(expected).collect();

    assert!(
        missing.is_empty() && extra.is_empty(),
        "completions/{script} is out of step with the CLI.\n  \
         missing (the CLI accepts these, the script does not offer them): {missing:?}\n  \
         extra (the script offers these, the CLI does not accept them): {extra:?}"
    );
}

#[test]
fn bash_completion_matches_the_cli() {
    let expected = flags_from_help();
    let found = long_flags(&read("completions/numbertext.bash"));

    assert_covers("numbertext.bash", &found, &expected);
}

#[test]
fn zsh_completion_matches_the_cli() {
    let expected = flags_from_help();
    let found = long_flags(&read("completions/_numbertext"));

    assert_covers("_numbertext", &found, &expected);
}

/// fish spells a long option `-l name` rather than `--name`, so this one
/// needs its own extraction rather than the shared scanner.
#[test]
fn fish_completion_matches_the_cli() {
    let script = read("completions/numbertext.fish");
    let found: BTreeSet<String> = script
        .split_whitespace()
        .collect::<Vec<_>>()
        .windows(2)
        .filter(|pair| pair[0] == "-l")
        .map(|pair| format!("--{}", pair[1]))
        .collect();

    assert_covers("numbertext.fish", &found, &flags_from_help());
}

/// The zsh file has to be named `_numbertext` for `#compdef` to work when
/// it is dropped on the `$fpath`, and it has to carry that directive.
#[test]
fn zsh_completion_is_installable_under_its_required_name() {
    let path = root().join("completions/_numbertext");

    assert!(path.is_file(), "zsh completions must be named _numbertext");
    assert!(
        read("completions/_numbertext").starts_with("#compdef numbertext"),
        "the #compdef directive must be the first line for zsh to autoload it"
    );
}

/// Every script resolves language codes at completion time instead of
/// baking in a list that a feature-selected build would contradict.
#[test]
fn no_script_hard_codes_the_language_list() {
    for script in [
        "completions/numbertext.bash",
        "completions/_numbertext",
        "completions/numbertext.fish",
    ] {
        let text = read(script);

        assert!(
            text.contains("--list"),
            "{script} must ask the binary for languages via --list"
        );
        // "pt-BR" is a regional code no build ever reports from --list;
        // finding it means someone pasted a language list in.
        assert!(
            !text.contains("pt-BR"),
            "{script} appears to hard-code language codes"
        );
    }
}

/// A guard on the extraction itself: if `long_flags` silently stopped
/// matching, every test above would pass against two empty sets.
#[test]
fn flag_extraction_actually_works() {
    let flags = long_flags("usage: --lang <code> --ordinal-number <n> -- not-a-flag");

    assert_eq!(
        flags.iter().map(String::as_str).collect::<Vec<_>>(),
        ["--lang", "--ordinal-number"]
    );
}
