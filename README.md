# numbertext-rs

[![CI](https://github.com/casara/numbertext-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/casara/numbertext-rs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/numbertext.svg)](https://crates.io/crates/numbertext)
[![docs.rs](https://img.shields.io/docsrs/numbertext)](https://docs.rs/numbertext)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A Rust library that spells out numbers as words — cardinals, ordinals,
gender/case variants, negatives, and currency amounts — in every language
shipped by [Numbertext.org](https://numbertext.github.io/).

Leia isto em [português (Brasil)](README.pt-BR.md).

## Why

[Numbertext.org](https://numbertext.github.io/) (upstream:
[libnumbertext](https://github.com/Numbertext/libnumbertext)) describes
number-to-text conversion for dozens of languages as small, declarative
rule files written in a tiny regex-rewriting DSL called **Soros**, rather
than as hand-written code per language. Adding a language — or a
regional or gender variant of one — means writing a new `.sor` file, not
writing new code.

This repository is a from-scratch Rust implementation of the Soros
interpreter (the [`soros`](crates/soros) crate) plus a friendly public
API (the [`numbertext`](src/lib.rs) crate), reusing the exact `.sor` rule
files from upstream (see [`data/`](data) and
[`data/SOURCE.md`](data/SOURCE.md)) unmodified.

## Install

As a library:

```sh
cargo add numbertext
```

As a command-line tool:

```sh
cargo install numbertext
```

That installs a `numbertext` binary built from this same crate, with
every bundled language:

```sh
numbertext --lang en --cardinal 12345    # twelve thousand three hundred forty-five
numbertext --lang pt-BR --ordinal 21     # vigésimo primeiro
numbertext --lang de --year 1999         # neunzehnhundertneunundneunzig
numbertext --list                        # every embedded language code
numbertext --help                        # every action and option
```

The binary costs library users nothing — Cargo does not build a
dependency's binary targets. It also honors the language features below,
so `cargo install numbertext --no-default-features --features lang-en`
gives you an English-only tool.

### Shell completions

[`completions/`](completions) has scripts for bash, zsh, and fish. They
are not generated at build time, so `cargo install` does not place them
for you — copy the one you need:

```sh
# bash
. completions/numbertext.bash                      # or source it from ~/.bashrc

# zsh — the file name matters, zsh autoloads it by it
mkdir -p ~/.zfunc && cp completions/_numbertext ~/.zfunc/
echo 'fpath=(~/.zfunc $fpath)' >> ~/.zshrc

# fish
cp completions/numbertext.fish ~/.config/fish/completions/
```

`--lang` completes from `numbertext --list` on the binary being
completed, so it offers exactly the languages *that* build embedded — an
English-only install completes only `en`, with no second list to keep in
step. The flag list is checked against the CLI's own `--help` by
`tests/completions.rs`, so a flag added to one and not the other fails
the build.

## Usage

```rust
use numbertext::Locale;

fn main() -> Result<(), numbertext::Error> {
    let en = Locale::new("en")?;

    println!("{}", en.cardinal("12345")); // twelve thousand three hundred forty-five
    println!("{}", en.ordinal("21"));     // twenty-first
    println!("{}", en.year("1999"));      // nineteen ninety-nine
    println!("{}", en.currency("USD", "2.50")); // two U.S. dollars and fifty cents

    let pt = Locale::new("pt")?;
    println!("{}", pt.cardinal("-5"));    // menos cinco

    Ok(())
}
```

### Two ways in

`Locale` resolves and compiles a language once. After that, **conversion
cannot fail** — every method returns a plain `String`, because the
underlying engine has no notion of a failed conversion (input no rule
matches produces an empty string). Prefer it whenever you convert more
than one number for the same language.

The free functions take the language code per call and fold the lookup
into the return type, for one-off use:

```rust
let n = numbertext::cardinal("de", "42")?; // "zweiundvierzig"
# Ok::<(), numbertext::Error>(())
```

Both go through the same process-wide cache, so neither recompiles a
language that has already been used.

### API

| `Locale` method | Free function | Purpose |
| --- | --- | --- |
| `cardinal(n)` | `cardinal(lang, n)` | "123" → "one hundred twenty-three" |
| `cardinal_i64(n)` | — | the same, for an `i64` |
| `ordinal(n)` | `ordinal(lang, n)` | "123" → "one hundred twenty-third" |
| `ordinal_number(n)` | `ordinal_number(lang, n)` | "123" → "123rd" |
| `year(n)` | `year(lang, n)` | "1999" → "nineteen ninety-nine" |
| `currency(iso, amount)` | `currency(lang, iso, amount)` | `NUMBERTEXT()`-style: "USD", "2.5" → "two U.S. dollars and fifty cents" |
| `money(iso, amount)` | `money(lang, iso, amount)` | `MONEYTEXT()`-style: "USD", "2.5" → "two and 50/100 U.S. dollars" |
| `help()` | `help(lang)` | the locale's self-documenting usage section |
| `convert(prefix, arg)` | `convert(lang, prefix, arg)` | the primitive above everything else — see below |
| `skipped_rules()` | — | rule lines that failed to compile (empty for every bundled locale) |
| — | `register_locale(code, source)` | add a language at runtime from raw `.sor` text |
| — | `languages()` | every currently known language code |
| — | `language_variants()` | the variant codes declared inside those files |

All numeric arguments are `&str` (`"123"`, `"-5"`, `"3.14"`), not
integers or floats, because the underlying engine is text-based and this
keeps arbitrary-precision numbers exact — a `.sor` file can spell out
numbers far past any fixed-width type. `cardinal_i64` covers the common
integer case.

`currency` and `money` are two different upstream-defined renderings of
the same amount, not interchangeable spellings. Pick the one your output
format actually expects.

### Regional variants (e.g. "en-GB", "pt-BR")

A regional code doesn't need — and won't have — its own `.sor` file:
`data/en.sor` and `data/pt.sor` each carry the base language's rules plus
a handful of lines tagged `[:en-GB:]`, `[:pt-BR:]` and so on for wording
that differs by region. Just pass the full code:

```rust
# use numbertext::Locale;
# fn main() -> Result<(), numbertext::Error> {
let gb = Locale::new("en-GB")?;
let us = Locale::new("en")?;

assert_eq!(gb.cardinal("101"), "one hundred and one");
assert_eq!(us.cardinal("101"), "one hundred one"); // no "and" — the default wording

let br = Locale::new("pt-BR")?;
let pt = Locale::new("pt")?;

assert_eq!(br.cardinal("16"), "dezesseis");
assert_eq!(pt.cardinal("16"), "dezasseis"); // European Portuguese default
# Ok(())
# }
```

`languages()` lists only codes backed by a file (`"en"`, `"pt"`, ...);
`language_variants()` lists the ones declared by tagged lines inside them
(`"en-GB"`, `"pt-BR"`, ...). The CLI's `--list` prints both, which is what
the shell completions read.

Neither list is exhaustive of what `Locale::new` *accepts*: any
`"<known-base>-<anything>"` resolves to the base file — `"en-ZZ"` gives
plain English rather than an error. What the lists contain is the set of
codes that actually change the output.

Two resolution rules are worth knowing, both matching upstream:

* A script variant is named with `_` in its file but may be written with
  either separator: `"hu_Hung"` and `"hu-Hung"` both give Old Hungarian.
* Norwegian is special-cased — `"nb"`, `"nn"`, `"nb-NO"` and `"nn-NO"`
  all read `no.sor`, which is where both written standards live.

### Gender, case, and other locale-specific variants

Numbertext's rule files aren't limited to a fixed cardinal/ordinal shape
— a language can define arbitrary named sections for whatever
grammatical variants it needs (`pt.sor` has `feminine`/`masculine`;
`ru.sor` has `cardinal-feminine`/`cardinal-neuter`; `it.sor` has
`ordinal-masculine`, and so on). These section names are **not**
standardized across languages, so this library does not pretend
otherwise with a one-size-fits-all `gender(...)` method. Call `convert`
with the section name you need:

```rust
# use numbertext::Locale;
# fn main() -> Result<(), numbertext::Error> {
let pt = Locale::new("pt")?;

assert_eq!(pt.convert("feminine", "2"), "duas");
# Ok(())
# }
```

Run `help(lang)`, or open the `.sor` file directly, to see which sections
a given language defines.

### Selecting which languages get embedded

By default, all 52 bundled `.sor` files are embedded into your binary,
regardless of which languages your program actually calls. If you only
need a few, opt into a smaller build with Cargo features — one
`lang-<code>` per language, where `<code>` is the `.sor` file's base name
lowercased with `_` becoming `-` (`en`, `pt`, `hu-hung`):

```toml
[dependencies]
numbertext = { version = "0.1", default-features = false, features = ["lang-en", "lang-pt"] }
```

That embeds only `en` and `pt`: `languages()` returns exactly
`["en", "pt"]`, both languages' regional variants (`en-GB`, `pt-BR`) keep
working since they live inside those same files, and calling
`cardinal("de", ...)` returns `Error::UnknownLanguage`. It saves roughly
236 KB of embedded rule data — measurable against the data, not against
your whole binary.

Note the failure mode: **it is silent at compile time and loud at
runtime.** Selecting the wrong set still builds; the language just isn't
there when you call it. If you add a call for a new language, add its
feature too.

`default-features = false` with no `lang-*` feature embeds nothing at all
(`languages()` returns an empty list). That's only useful if you supply
every locale yourself via `register_locale`.

Cargo features are additive across a dependency graph, which behaves
sensibly here: if two crates in your build ask for different language
sets, you get the union, never a missing language.

### Adding a new language

Because the whole library is data-driven, you don't need to touch any
Rust code to add a language:

```rust
let source = "1 uno\n2 dos\n3 tres\n"; // a real file needs far more rules
numbertext::register_locale("es-mini", source)?;

assert_eq!(numbertext::cardinal("es-mini", "2")?, "dos");
# Ok::<(), numbertext::Error>(())
```

or add a file under [`data/`](data) — see
[`data/SOURCE.md`](data/SOURCE.md#adding-a-new-language) for a short
guide and a link to the upstream Soros language specification.

## The Soros interpreter

The [`soros`](crates/soros) crate is the language interpreter on its own:
no notion of numbers, currencies, or languages. It's a separate crate
rather than a private module so that the boundary is enforced by Cargo
instead of by convention — `soros` has no dependency on `numbertext` to
reach back through. It is usable directly if you have some other use for
the language:

```rust
let program = soros::Program::compile(r"(.*)(.) \2$1", "");

assert_eq!(program.run("hello"), "olleh");
```

Patterns compile through [`fancy-regex`](https://docs.rs/fancy-regex),
which uses the linear-time [`regex`](https://docs.rs/regex) engine for
any pattern without a backreference or lookaround, and a
step-bounded backtracking engine for the rest — which some real locale
files genuinely need (`hu.sor` looks grammatical suffixes up with a
backreference inside the matching pattern). See
[AGENTS.md](AGENTS.md#regex-engine-one-crate-not-two-glued-together) for
the full rationale.

## API stability

`numbertext` is released as **v0.x**, and in Semantic Versioning that
carries a specific meaning worth stating plainly rather than leaving you
to infer it: **a minor release may break the public API.** Pin a version,
read the [CHANGELOG](CHANGELOG.md) before upgrading, and expect to make
small mechanical changes when you do.

What counts as the public API: every public item in the `numbertext` and
`soros` crates, **and the command-line flags of the `numbertext`
binary** — once a tool is installable, the flags are a contract whether
or not anyone says so, so this says so.

Not public: the exact wording of error messages, of `--help` output, and
of `help()` output (which is upstream prose and changes with a data
sync). `Error`'s variants — meant to be matched — are the stable part of
error handling; message text is not.

The MSRV is **Rust 1.85**, and CI tests against exactly that version as
well as `stable`, so the declaration stays honest. Raising it is a
breaking change and gets a changelog entry.

Breaking changes are listed under `### Changed` or `### Removed` in the
changelog, with the migration in the same entry.

## Skill for AI assistants

[skills/using-numbertext/SKILL.md](skills/using-numbertext/SKILL.md)
documents, in a format AI tools can load on demand (Claude Skill), how to
use `numbertext` idiomatically: `Locale` versus the free functions, the
`convert` primitive, regional variants, gender/case sections, and
selective language embedding. To use it in a project that depends on
`numbertext`, copy the directory into it:

```sh
cp -r skills/using-numbertext <your-project>/.claude/skills/using-numbertext
```

## Development

`make help` lists every command. Before opening a PR:

```sh
make check     # fmt-check + lint + msrv + spell + tests (incl. doctests) + feature selection
make lint-md   # markdown lint (requires Node.js >= 20)
```

Style conventions are documented in
[docs/coding-style.md](docs/coding-style.md).

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full contribution workflow
(commit message convention, branch/PR process) and
[CLAUDE.md](CLAUDE.md)/[AGENTS.md](AGENTS.md) for a deeper architecture
overview aimed at contributors and coding agents.

## License

The Rust code in this repository is MIT-licensed, see [LICENSE](LICENSE).

The `.sor` rule files under [`data/`](data) are a different matter: they
are copied unmodified from upstream libnumbertext and remain under its
**BSD-3-Clause** license, see [`data/LICENSE`](data/LICENSE) and
[`data/SOURCE.md`](data/SOURCE.md). Both licenses travel with the crate,
since the rule files are embedded in it.
