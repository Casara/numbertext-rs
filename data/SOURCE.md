# Data source

The `.sor` files in this directory are copied verbatim from the `data/`
directory of the upstream
[libnumbertext](https://github.com/Numbertext/libnumbertext) project (the
reference implementation behind
[Numbertext.org](https://numbertext.github.io/)), with one documented
exception - see [Local patches](#local-patches-pending-upstream) below.

- Upstream repository: <https://github.com/Numbertext/libnumbertext>
- Synced commit: `a4b0225813b015a0f796754bc6718be20dd9943c`
- Upstream `VERSION` file: `1.0.11`
- Sync date: 2026-08-26
- Files: 52 `*.sor` locales, plus `LICENSE` (upstream's `COPYING`)
- Verification: every local `*.sor` file's git blob SHA1
  (`git hash-object`) matches the blob SHA the GitHub Contents API
  reports for the commit above, **except `bg.sor`**, which carries the
  one-line patch documented below.

## Local patches (pending upstream)

### `bg.sor` line 146: transposed `]`/`)` in an ordinal-thousands rule

```diff
- ([0-9]{1,3})([0-9]{3}) $(f:|$1) хиляди[ $(and:$(ordinal \2)])
+ ([0-9]{1,3})([0-9]{3}) $(f:|$1) хиляди[ $(and:$(ordinal \2))]
```

Upstream's line ends `\2)])` - the closing bracket before the second
closing paren - so the `[...]` block and the `$(...)` call cross rather
than nest. The intended form closes both parens before the bracket:
`\2))]`.

That this is a typo rather than a deliberate construct is verifiable by
inspection, not a guess about intent. The singular counterpart one line
above, `1([0-9]{3}) хиляда[ $(and:$(ordinal \1))]`, already nests
correctly, and the general shape every other magnitude uses -
`$(eN:\1)[ $(and:$2)]` - is the same again. Both rules build the same
"N thousand [and REMAINDER-th]" output. It reads as a two-character
transposition.

The reference implementations do not notice, and the reason is worth
stating exactly rather than in the abstract. They rewrite `[...]` into
call syntax with a plain text substitution and then replace **every**
`]` with `)`, unconditionally, before any paren matching happens:

- C++ [`src/Soros.cxx:125`](https://github.com/Numbertext/libnumbertext/blob/master/src/Soros.cxx#L125):
  `s2 = translate(s2, L"]", L")", L"");`
- Python [`src/Soros.py:75`](https://github.com/Numbertext/libnumbertext/blob/master/src/Soros.py#L75):
  `s2 = re.sub(r"]", ")", s2)`

So a bracket never takes part in a nesting analysis at all - there is
nothing for the crossing to be inconsistent with, and the malformed line
happens to come out balanced. That is an implementation accident, not a
documented semantic: no specification says `]` and `)` are
interchangeable. This port parses the template into a tree instead,
which is what surfaces the typo - and why the fix belongs in the data.

Patching the *interpreter* to tolerate crossing spans was considered and
rejected: there is no principled tree for a genuine crossing, so
building one would mean either picking an arbitrary resolution or
permanently encoding another implementation's pass ordering into this
one.

**Status**: reported upstream, still open - see
[Numbertext/libnumbertext#139](https://github.com/Numbertext/libnumbertext/issues/139)
(confirmed open and unmerged as of the sync date above; `master` still
carries `\2)])`). Once it is merged and picked up by a
`scripts/sync-data.sh` run, remove this section and the matching
re-patch step in that script - the script prints a reminder when it
notices the patch is no longer needed.

## License

These files are distributed under the upstream project's BSD-3-Clause
license, reproduced in [LICENSE](LICENSE) (upstream calls the file
`COPYING`). That is a **different license** from the one covering the
Rust code in the rest of this repository, which is MIT - see the
top-level [LICENSE](../LICENSE). Only the contents of this `data/`
directory are BSD-3-Clause.

## Updating

Run `scripts/sync-data.sh` from the repository root. It re-downloads
every `*.sor` file and the license from upstream `master`, prints the
commit it synced to, and reapplies the `bg.sor` patch above (or reports
that upstream already has the fix, in which case remove the patch step
and the section above).

Then:

1. Review `git diff data/`.
2. Update the commit hash, `VERSION`, and date recorded above.
3. Run `make gen-features` if a locale was added or removed - the
   per-language Cargo features are generated from this directory's file
   list - then `make test-feature-matrix`, which builds each `lang-*`
   feature on its own. A locale whose name does not survive the mangling
   to a feature name (lowercased, `_` to `-`) would otherwise ship a
   feature nothing can enable, and no other check looks at that.
4. Run `make test`. `tests/smoke.rs` is the one that matters here: it
   asserts every bundled locale compiles with **zero** skipped rules, so
   a sync that introduces a malformed rule fails loudly instead of
   silently degrading one language's wording.

## Adding a new language

Numbertext.org's whole design point is that a new language needs a new
`.sor` rule file, not code changes. To add one:

1. Read the
   [Soros language specification](https://github.com/Numbertext/libnumbertext/blob/master/doc/sorosspec.pdf),
   or study an existing file such as `en.sor`, `pt.sor`, or `de.sor`.
2. Write `data/<code>.sor`, then run `make gen-features` so the new
   locale gets its `lang-<code>` Cargo feature. Alternatively, use
   `numbertext::register_locale` to add it at runtime without touching
   this repository at all.
3. Add a test asserting a handful of known conversions, similar to the
   English and Portuguese cases in `tests/numbertext.rs` - and run the
   code to get those values rather than guessing them.
4. Consider contributing the file back to upstream `libnumbertext` too,
   so the rest of the Soros ecosystem benefits from it.
