# Security Policy

## Supported Versions

Before `v1.0.0`, only the latest released version receives security
fixes. Once `v1.0.0` ships, this section will list which major versions
still get patches.

## Threat model

`numbertext` has no network, filesystem, or authentication surface of its
own. Converting a number with a bundled locale does not read or write
anything. Two things are worth being deliberate about:

### `register_locale` compiles caller-supplied text into regular expressions

A `.sor` source is a program. Compiling one builds regular expressions
from it and produces a recursive evaluator, so treat it as configuration
you control and review — not as untrusted end-user input.

Two bounds apply, and both are mitigations rather than immunity:

- **Backtracking is bounded.** Patterns compile through
  [`fancy-regex`](https://docs.rs/fancy-regex), which uses the
  linear-time [`regex`](https://docs.rs/regex) engine for any pattern
  without a backreference or lookaround — which is nearly every rule.
  Only the rest can backtrack at all, and each match is capped at
  `soros::BACKTRACK_LIMIT` steps, so a pathological pattern degrades to
  "this rule doesn't match" instead of hanging. Being a step count
  rather than a wall-clock timeout, the cap behaves identically on every
  machine.
- **Recursion is bounded.** Evaluation is recursive, and
  `soros::MAX_RECURSION_DEPTH` caps it. This matters more in Rust than
  in a language with growable stacks: an unbounded recursive evaluator
  meeting a rule file that does not converge would overflow the stack,
  which aborts the process rather than returning an error the caller
  could handle. With the cap, the branch yields an empty string and the
  caller stays alive.

### Conversion input

`cardinal`/`ordinal`/`year`/`currency`/`money`/`convert` take arbitrary
strings, and that is fine: the bundled rule files are bounded in
practice, and the two limits above apply regardless of which locale is
in use. Input that matches no rule yields an empty string.

## Reporting a Vulnerability

Please **do not** open a public issue for a security vulnerability.

Use GitHub's private vulnerability reporting instead: go to the
[Security tab](https://github.com/casara/numbertext-rs/security) of this
repository and select "Report a vulnerability." That opens a private
advisory visible only to the maintainers until a fix is ready.

Include, if possible:

- The affected version or commit.
- A minimal reproduction — the exact call and input that triggers it.
- The impact you believe it has.

We'll acknowledge the report and work with you on a disclosure timeline
before any public advisory is published.
