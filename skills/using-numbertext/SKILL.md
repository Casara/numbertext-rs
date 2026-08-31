---
name: using-numbertext
description: >-
  Guidance for writing Rust that uses the numbertext crate - spelling out
  numbers, ordinals, years, currency amounts, and gender/case variants in
  any of its bundled languages, plus registering a new one at runtime and
  shrinking a build by selecting which locales get embedded. Use when the
  project depends on numbertext, or when asked to add or change a
  number-to-text conversion, add a language, or cut embedded locale data.
---

# Using numbertext

`numbertext` spells out numbers as words — cardinals, ordinals, years,
currency amounts, and gender/case variants — in every language shipped by
[Numbertext.org](https://numbertext.github.io/). This skill is for code
that *consumes* numbertext as a dependency, not for numbertext's own
source; if you're working inside the `numbertext-rs` repository itself,
follow its `AGENTS.md` instead.

This file is meant to be copied into any project that depends on
numbertext, so it never lives next to numbertext's own source tree. Paths
mentioned below (`data/*.sor`, `AGENTS.md`) are links into the
[numbertext-rs repository](https://github.com/casara/numbertext-rs),
not paths relative to this file.

## Reach for `Locale`, not the free functions

There are two entry points, and the choice matters more than it looks.

```rust
use numbertext::Locale;

let en = Locale::new("en")?;             // resolves and compiles once; can fail

let c = en.cardinal("123");              // "one hundred twenty-three"  - String, not Result
let o = en.ordinal("123");               // "one hundred twenty-third"
let n = en.ordinal_number("123");        // "123rd"
let y = en.year("1999");                 // "nineteen ninety-nine"
let m = en.currency("USD", "2.5");       // "two U.S. dollars and fifty cents"
let k = en.money("USD", "2.5");          // "two and 50/100 U.S. dollars"
let h = en.help();                       // the locale's own usage summary
# Ok::<(), numbertext::Error>(())
```

**Conversion returns `String`, not `Result`.** Only *resolving* a
language can fail. The Soros engine underneath has no notion of a failed
conversion: input that matches no rule yields an empty string. So resolve
once, handle that one error, and convert freely.

The free functions take the language per call and fold the lookup into
the return type. Use them for a genuinely one-off conversion:

```rust
let c = numbertext::cardinal("de", "42")?;   // "zweiundvierzig"
# Ok::<(), numbertext::Error>(())
```

They are exactly `Locale::new(lang)?.method(..)`. In a loop, hoist the
`Locale` out — not for the compile cost (that is cached process-wide)
but because the `Result` per call is noise.

## Numbers are strings

Every numeric argument is `&str` (`"123"`, `"-5"`, `"3.14"`), not an
integer or float. The engine is text-based, and a rule file can spell out
numbers far past any fixed-width type, so text keeps arbitrary precision
exact. `Locale::cardinal_i64` is the convenience for the common integer
case; there is no float equivalent on purpose — format the float yourself
so the rounding is your decision, not the library's.

## `currency` and `money` are not synonyms

They are two different upstream-defined renderings of the same amount:
`currency` is the `NUMBERTEXT()`-compatible form, `money` the
`MONEYTEXT()`-compatible one, which several locales format quite
differently (a fraction for the sub-unit remainder instead of a second
currency name). Pick the one your output format expects. Don't assume
they agree.

## `year` is not `cardinal`

Also worth not assuming:

```rust
# let en = numbertext::Locale::new("en").unwrap();
assert_eq!(en.year("1999"), "nineteen ninety-nine");
assert_eq!(en.cardinal("1999"), "one thousand nine hundred ninety-nine");
```

## Regional variants (e.g. "en-GB", "pt-BR")

A regional code has no rule file of its own: the base language's file
carries lines tagged for that region, and passing the full code activates
them while falling back to the base file for everything else.

```rust
# use numbertext::Locale;
# fn main() -> Result<(), numbertext::Error> {
assert_eq!(Locale::new("en-GB")?.cardinal("101"), "one hundred and one");
assert_eq!(Locale::new("en")?.cardinal("101"), "one hundred one");
# Ok(())
# }
```

`languages()` lists **only codes with a file of their own**;
`language_variants()` lists the ones declared by tagged lines inside
them (`en-GB`, `pt-BR`, `Roman-large`, ...). Validating user input
against `languages()` alone is a common mistake — check both, or just
try `Locale::new` and handle the error.

Neither list bounds what is accepted: `Locale::new("en-ZZ")` succeeds
and returns plain English, because any unknown suffix falls back to the
base file. The lists are the codes that *change the output*, not the
codes that work.

Two resolution rules match upstream and are easy to get wrong:

- A script variant may be written with either separator — `"hu_Hung"`
  and `"hu-Hung"` both give Old Hungarian. (A port that only handles the
  underscore returns ordinary Hungarian *silently*.)
- Norwegian is special-cased: `"nb"`, `"nn"`, `"nb-NO"`, `"nn-NO"` all
  read `no.sor`, which carries both written standards.

## Gender, case, and other locale-specific sections

A rule file isn't limited to a fixed cardinal/ordinal shape; it can
define arbitrary named sections (`pt.sor` has `feminine`/`masculine`;
`ru.sor` has `cardinal-feminine`/`cardinal-neuter`; `it.sor` has
`ordinal-masculine`). **These names are not standardized across
languages.** There is no `gender()` method, and guessing a name that
"should" exist for a language you haven't checked is the most common
mistake with this crate. Use `convert`, and confirm the section exists
first via `help(lang)` or by reading that language's `.sor` file:

```rust
# let pt = numbertext::Locale::new("pt").unwrap();
assert_eq!(pt.convert("feminine", "2"), "duas");
```

`convert(prefix, arg)` builds one input string: `arg` alone when `prefix`
is empty, `prefix` alone when `arg` is empty, and `"{prefix} {arg}"`
otherwise. Every named method is a thin wrapper over it.

## Errors

`numbertext::Error` is a `#[non_exhaustive]` enum. Match on variants, not
on message text — the wording is explicitly not part of the API.

```rust
match numbertext::cardinal(user_lang, n) {
    Ok(text) => text,
    Err(numbertext::Error::UnknownLanguage { .. }) => fallback(),
    Err(err) => return Err(err),
}
```

- `Error::UnknownLanguage { lang }` — the code, and for a regional
  variant its base language too, has no bundled or registered source.
  This is the one every call site taking a caller-supplied language code
  should handle.
- `Error::EmptyLocaleCode` — `register_locale` was called with `""`.

Because the enum is `#[non_exhaustive]`, always include a catch-all arm.

## Adding a language at runtime

The library is data-driven, so a consumer can add a language it doesn't
bundle without forking or waiting for a release:

```rust
numbertext::register_locale("es-mini", sor_source)?;  // sor_source: a ".sor" file's contents
# Ok::<(), numbertext::Error>(())
```

Do this once at startup, before any call that might use the code.
Registering a code that already exists replaces it. A `Locale` already
resolved from the previous source is an immutable snapshot and keeps
working with what it compiled — resolve a new one to pick up the
replacement.

Registration does not validate the whole program: a malformed individual
rule is *skipped*, not rejected, so a typo shows up as wording that
silently falls through to another rule. If you supply your own source,
check it:

```rust
# numbertext::register_locale("mine", "1 uno\n").unwrap();
let locale = numbertext::Locale::new("mine")?;
assert!(locale.skipped_rules().is_empty(), "{:?}", locale.skipped_rules());
# Ok::<(), numbertext::Error>(())
```

Treat `sor_source` as configuration you control, not as untrusted
end-user input: it compiles to regular expressions and drives a recursive
evaluator. Both are bounded (`soros::BACKTRACK_LIMIT`,
`soros::MAX_RECURSION_DEPTH`), but a bound is a mitigation, not a licence
to load hostile input. See numbertext's `SECURITY.md`.

## Shrinking the embedded locale data

By default every bundled `.sor` file (52 languages) is embedded,
regardless of which ones the program calls. To embed only some:

```toml
[dependencies]
numbertext = { version = "0.1", default-features = false, features = ["lang-en", "lang-pt"] }
```

The feature name is `lang-` plus the `.sor` base name lowercased with `_`
becoming `-`: `lang-en`, `lang-pt`, `lang-hu-hung`. It saves roughly
236 KB of embedded data. Regional variants of a selected language keep
working, since they live inside its file.

**The failure mode is silent at compile time and loud at runtime.** An
untagged language still builds; it returns `Error::UnknownLanguage` when
called. If you add a call for a new language, add its feature in the same
change. `default-features = false` with no `lang-*` feature embeds
nothing at all — useful only if every locale comes from
`register_locale`.

Cargo features are additive across a dependency graph, so if two crates
in the build request different sets, the result is the union. You cannot
accidentally lose a language another crate needs.

## Reference

Full rationale — the Soros rule-file format, the regex engine's
delegation and its bounds, and why gender/case sections aren't
standardized — lives in the numbertext-rs repository: `README.md` (usage
and API stability), `AGENTS.md` (architecture and the Soros language),
and `data/SOURCE.md` (adding a language as a data file). API docs are on
[docs.rs/numbertext](https://docs.rs/numbertext).
