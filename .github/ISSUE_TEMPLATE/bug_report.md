---
name: Bug report
about: Something in numbertext isn't behaving as documented
title: ''
labels: bug
assignees: ''
---

**Version**
The `numbertext` version or commit, and `rustc --version`.

**What happened**
A clear description of the behavior you saw, including the exact output.

**What you expected**
What you expected instead. If it's about a specific language's wording,
say whether you checked it against the actual `data/<code>.sor` file or
against [numbertext.github.io](https://numbertext.github.io/) — locale
files encode real, sometimes surprising stylistic choices (see AGENTS.md's
"Verify locale output before writing it down"), so a wording difference
isn't necessarily a bug in this port.

**Minimal reproduction**
The exact call that reproduces it.

```rust
// e.g. numbertext::cardinal("en", "1001")
```

**Feature selection**
If you build with `default-features = false`, list the `lang-*` features
you enabled — a missing language is the expected result of an incomplete
selection, not a bug.

**Additional context**
Backtrace, logs, anything else relevant.
