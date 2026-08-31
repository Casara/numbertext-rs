---
name: Feature request
about: Propose something new for numbertext
title: ''
labels: enhancement
assignees: ''
---

**Problem**
What are you trying to do that `numbertext` doesn't support today?
Prefer a concrete scenario over an abstract one.

**Proposed solution**
What you'd like it to do. If this is a new language or regional variant,
note whether it can be added purely as a `data/<code>.sor` file (see
[data/SOURCE.md](../../data/SOURCE.md#adding-a-new-language)) or at
runtime via `register_locale` — that's almost always true, and doesn't
need a release.

**Alternatives considered**
Anything else you tried, including `register_locale` with your own `.sor`
source as a workaround.

**Soros spec, if relevant**
If this touches the interpreter itself (`crates/soros`), point at the
relevant part of the
[Soros language specification](https://github.com/Numbertext/libnumbertext/blob/master/doc/sorosspec.pdf).
See AGENTS.md first for the design decisions already settled — the regex
engine's structure and the `bg.sor` data patch each have their reasoning
written down.
