//! A small command-line front end for the `numbertext` crate, for quick
//! manual checks and as a runnable usage example.
//!
//! The argument parsing here is hand-rolled rather than done with a
//! parsing crate on purpose. `numbertext` is a library, and a binary
//! target's dependencies are the package's dependencies: pulling one in
//! for this would put it in the tree of everyone who depends on the
//! library and never runs the binary. The flag set is small and stable
//! enough that the trade is not close.

use std::process::ExitCode;

use numbertext::Locale;

const USAGE: &str = "\
usage: numbertext [--lang <code>] <action>

actions:
  --cardinal <n>          spell out a cardinal number      (123 -> one hundred twenty-three)
  --ordinal <n>           spell out an ordinal number      (123 -> one hundred twenty-third)
  --ordinal-number <n>    format an abbreviated ordinal    (123 -> 123rd)
  --year <n>              spell out a calendar year        (1999 -> nineteen ninety-nine)
  --currency <ISO> <n>    spell out a currency amount      (USD 2.5 -> two U.S. dollars and fifty cents)
  --money <ISO> <n>       the MONEYTEXT-style variant      (USD 2.5 -> two and 50/100 U.S. dollars)
  --convert <section> <n> call a locale-specific section   (pt: feminine 2 -> duas)
  --help-section          print the locale's own usage summary
  --list                  list available language codes, variants included
  --help                  print this message

options:
  --lang <code>           language code, default \"en\" (see --list)

A code with no rule file of its own, such as \"en-GB\" or \"pt-BR\", works
too: it reads its base language's file and activates that region's lines.
";

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(Some(output)) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Ok(None) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("numbertext: {message}");
            eprintln!("\n{USAGE}");
            ExitCode::FAILURE
        }
    }
}

/// Every code `--lang` accepts and that changes the output: the bundled
/// languages plus the variants their rule files declare.
///
/// The two are separate in the library (`languages` and
/// `language_variants`) because they are different things - one is
/// backed by a file, the other by tagged lines inside one. They are
/// merged here because the question a CLI user is asking is "what can I
/// pass to --lang", and because this is what shell completion reads.
fn language_codes() -> Vec<String> {
    let mut codes = numbertext::languages();
    codes.extend(numbertext::language_variants());
    codes.sort();

    codes
}

/// Runs one action, returning what to print (or `None` when the action
/// printed for itself).
///
/// # Errors
///
/// A human-readable message for an unusable command line or an unknown
/// language.
fn run(args: Vec<String>) -> Result<Option<String>, String> {
    let mut lang = "en".to_owned();
    let mut action: Option<(String, Vec<String>)> = None;
    let mut rest = args.into_iter();

    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                println!("{USAGE}");
                return Ok(None);
            }
            "--list" => {
                return Ok(Some(language_codes().join("\n")));
            }
            "--lang" => {
                lang = rest.next().ok_or("--lang needs a language code")?;
            }
            "--help-section" => action = Some((arg, Vec::new())),
            "--cardinal" | "--ordinal" | "--ordinal-number" | "--year" => {
                let value = rest.next().ok_or_else(|| format!("{arg} needs a number"))?;
                action = Some((arg, vec![value]));
            }
            "--currency" | "--money" | "--convert" => {
                let first = rest
                    .next()
                    .ok_or_else(|| format!("{arg} needs two arguments"))?;
                let second = rest
                    .next()
                    .ok_or_else(|| format!("{arg} needs two arguments"))?;
                action = Some((arg, vec![first, second]));
            }
            other => return Err(format!("unrecognized argument {other:?}")),
        }
    }

    let (name, values) = action.ok_or("no action given")?;
    let locale = Locale::new(&lang).map_err(|err| err.to_string())?;

    let output = match (name.as_str(), values.as_slice()) {
        ("--cardinal", [n]) => locale.cardinal(n),
        ("--ordinal", [n]) => locale.ordinal(n),
        ("--ordinal-number", [n]) => locale.ordinal_number(n),
        ("--year", [n]) => locale.year(n),
        ("--currency", [iso, amount]) => locale.currency(iso, amount),
        ("--money", [iso, amount]) => locale.money(iso, amount),
        ("--convert", [section, arg]) => locale.convert(section, arg),
        ("--help-section", []) => locale.help(),
        _ => unreachable!("every action parsed above is handled here"),
    };

    Ok(Some(output))
}
