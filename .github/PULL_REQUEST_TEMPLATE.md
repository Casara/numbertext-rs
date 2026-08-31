<!-- markdownlint-disable-next-line MD041 -- GitHub PR templates aren't rendered as a titled page, no H1 needed. -->
## What and why

<!-- What changes, and the motivation - link an issue if there is one. -->

## Checklist

- [ ] `make check` passes locally (fmt-check + lint + msrv + tests
      including doctests + the feature-selection builds).
- [ ] `make lint-md` passes, if any Markdown file changed.
- [ ] New or changed behavior has a test. For a new language, a handful
      of known-correct conversions (see `tests/numbertext.rs`) -
      verified by actually running the code, not guessed (see AGENTS.md's
      "Verify locale output before writing it down").
- [ ] Commit messages follow [CONTRIBUTING.md](../CONTRIBUTING.md)'s
      Conventional Commits convention.

### If a public item changed

<!-- Delete this section if the change is internal only. -->

- [ ] `README.md` **and** `README.pt-BR.md` - prose, the API table, and
      examples still match. Both sides, or the pair contradicts itself.
- [ ] Doc comments updated; their examples still assert the real output.
- [ ] `AGENTS.md`, if an invariant or design decision changed
      (`CLAUDE.md` just imports it).
- [ ] `CHANGELOG.md` - and if this is a breaking change, read
      [Changing the public API](../CONTRIBUTING.md#changing-the-public-api).
- [ ] `skills/using-numbertext/SKILL.md`, if the API it documents moved.
- [ ] `make doc-sync` - lists what a public-API change may have left
      stale; it never fails the build, so check it by hand.

### If `data/*.sor` changed

<!-- Delete this section otherwise. -->

- [ ] Came from `scripts/sync-data.sh` (an upstream sync), not a hand
      edit - see AGENTS.md's "Don't hand-edit data/*.sor".
- [ ] `data/SOURCE.md`'s synced commit, `VERSION`, and date updated.
- [ ] `make check-data` passes.
- [ ] `make gen-features` re-run if a locale was added or removed, and
      `make test-feature-matrix` after it - that is the check that a new
      locale's file name actually round-trips into a feature anyone can
      enable.
