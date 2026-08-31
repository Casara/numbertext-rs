# Dev tools, version-pinned. They are installed on demand into the cargo
# bin directory rather than declared as dependencies: numbertext is a
# library, and nothing a linter or an auditor needs belongs in the
# dependency tree of whoever depends on it.
CARGO_DENY    := cargo-deny@0.20.2
CARGO_MUTANTS := cargo-mutants@27.1.0
CARGO_NEXTEST := cargo-nextest@0.9.143
CARGO_HACK    := cargo-hack@0.6.45
SEMVER_CHECKS := cargo-semver-checks@0.50.0
TYPOS         := typos-cli@1.49.1
CARGO_MSRV    := cargo-msrv@0.19.3
CARGO_MACHETE := cargo-machete@0.9.2
COCOGITTO     := cocogitto@7.0.0

# The one non-Rust dev tool here, run through npx instead of a global
# install. Requires Node.js >= 20.
MARKDOWNLINT := markdownlint-cli2@0.23.2

# The MSRV declared in Cargo.toml. Kept here as well so `make msrv` can
# check the two against each other instead of trusting a comment.
MSRV := 1.85

# What cargo already has, as `name@version` entries. `cargo install`
# re-resolves the registry even when the pinned version is present, which
# turned a routine `make spell` into a multi-minute wait; this makes the
# install a no-op once the right version is in place, while still picking
# up a changed pin above.
INSTALLED := $(shell cargo install --list 2>/dev/null | sed -n 's/^\([a-z][^ ]*\) v\([^:]*\):$$/\1@\2/p')
ensure = $(if $(filter $(1),$(INSTALLED)),true,cargo install --locked $(1))

# `cargo install` puts binaries next to the active cargo. Under a version
# manager that directory is not on PATH until the manager refreshes its
# shims (with asdf: `asdf reshim rust`), so a tool the target just
# installed would fail on the same run with "command not found". Cargo
# subcommands are unaffected - cargo finds those itself - but `typos` and
# `cog` are plain binaries. Resolving the directory here removes the
# manual step, and changes nothing where PATH is already correct.
#
# Appended, never prepended: that directory can also hold a `cargo` of its
# own (a rustup proxy with no default toolchain, under asdf), and putting
# it first shadows the working one and breaks every cargo target.
CARGO_BIN := $(firstword $(wildcard $(shell asdf where rust 2>/dev/null)/bin $(HOME)/.cargo/bin))
export PATH := $(PATH)$(if $(CARGO_BIN),:$(CARGO_BIN),)

# `make` with no target prints the command list rather than silently
# building something. Discovering what a repository can do should not
# require reading its Makefile.
.DEFAULT_GOAL := help

.PHONY: help build run fmt fmt-check lint lint-fix lint-md audit doc \
	test test-nextest test-features test-feature-matrix test-msrv \
	test-mutation test-mutation-dry-run semver-checks coverage watch \
	gen-features sync-data check-data doc-sync msrv check clean

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-22s\033[0m %s\n", $$1, $$2}'

build: ## Build the workspace
	@cargo build --workspace --all-targets

run: ## Run the CLI (pass flags via ARGS, e.g. make run ARGS="--lang en --cardinal 42")
	@cargo run --quiet --bin numbertext -- $(ARGS)

watch: ## Rebuild and re-run the tests on every change (needs cargo-watch)
	@cargo watch -x 'test --workspace --all-targets'

fmt: ## Format the code
	@cargo fmt --all

fmt-check: ## Check formatting without changing anything
	@cargo fmt --all --check

lint: ## Run clippy over every target and feature, warnings denied
	@cargo clippy --workspace --all-targets --all-features -- -D warnings

lint-fix: ## Run clippy and apply the fixes it can make automatically
	@cargo clippy --workspace --all-targets --all-features --fix --allow-staged

lint-md: ## Lint markdown files (requires Node.js >= 20)
	@npx --yes $(MARKDOWNLINT) "**/*.md" "#NOTES.md" "#TEMP.md" "#TEMP" "#target"

spell: ## Spell-check source and prose (skips data/, see _typos.toml)
	@$(call ensure,$(TYPOS))
	@typos

unused-deps: ## Report dependencies declared but never used
	@$(call ensure,$(CARGO_MACHETE))
	@cargo machete

commit-check: ## Lint commit messages against the Conventional Commits convention
# Reads cog.toml, which declares the three types CONTRIBUTING.md adds to
# the conventional set (cleanup, remove, raw). Passing RANGE limits the
# check, e.g. RANGE=origin/main..HEAD for just this branch.
# A fresh clone with no commits yet has nothing to lint, and cog exits
# non-zero on the missing ref. That is not a failure worth propagating.
	@$(call ensure,$(COCOGITTO))
	@git rev-parse --verify --quiet HEAD >/dev/null 2>&1 \
		|| { echo "no commits yet - nothing to check"; exit 0; }; \
	cog check $(RANGE)

audit: ## Check advisories, licenses, and banned/duplicate dependencies
# cargo-deny's `advisories` check is what cargo-audit does; running deny
# gets the license and dependency-ban policy in the same pass, against
# the same lockfile, instead of maintaining two tools that overlap.
	@$(call ensure,$(CARGO_DENY))
	@cargo deny check

doc: ## Build the API documentation the way docs.rs does
	@RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps

test: ## Run every test, including the doc examples
	@cargo test --workspace --all-targets
	@cargo test --workspace --doc

test-nextest: ## Run the tests through cargo-nextest (faster; does NOT run doctests)
# nextest runs each test in its own process, which is both faster here
# and stricter: the locale registry is process-wide shared state, so a
# test that only passes because an earlier test in the same process
# populated it fails under nextest and passes under `cargo test`.
# It cannot run doctests at all, hence the second line - dropping it
# would silently stop checking 39 documentation examples.
	@$(call ensure,$(CARGO_NEXTEST))
	@cargo nextest run --workspace --all-features
	@cargo test --workspace --doc

test-features: ## Sanity-check selective language embedding (see README)
# A build that keeps only "en" and "pt" must still convert both, and must
# report every other language as unknown rather than silently falling
# back - proof that opting in shrinks the data without changing behavior
# for the languages you kept.
	@cargo test --quiet --no-default-features --features lang-en,lang-pt --test features
	@cargo test --quiet --no-default-features --test features

test-feature-matrix: ## Check that every lang-* feature builds on its own (slow)
# There are 50+ language features and `make check` only exercises three
# configurations of them. This is the one that would catch a locale whose
# file name does not round-trip through gen-locale-features.py's
# name mangling - the failure mode that produces a feature nothing can
# ever enable. Deliberately not part of `check`: it is dozens of builds.
	@$(call ensure,$(CARGO_HACK))
	@cargo hack check --each-feature --no-dev-deps

test-msrv: ## Run the tests on the declared MSRV toolchain
	@rustup toolchain install $(MSRV) --profile minimal 2>/dev/null || true
	@cargo +$(MSRV) test --workspace --all-targets
	@cargo +$(MSRV) test --workspace --doc

test-mutation: ## Run mutation tests (writes mutants.out/)
# --workspace, because cargo-mutants otherwise only mutates the package
# in the current directory. --test-workspace, because it otherwise runs
# only the mutated package's own tests: a mutation in `soros` that only
# the `numbertext` crate's tests would catch would be reported as a
# surviving mutant that nothing covers. Both flags are the difference
# between a verdict worth reading and a misleading one.
# --test-tool=nextest because mutation testing runs the suite once per
# mutant, so the per-run saving is multiplied by hundreds.
	@$(call ensure,$(CARGO_MUTANTS))
	@$(call ensure,$(CARGO_NEXTEST))
	@cargo mutants --workspace --test-workspace=true --test-tool=nextest

test-mutation-dry-run: ## List the mutants without testing them (much faster)
	@$(call ensure,$(CARGO_MUTANTS))
	@cargo mutants --workspace --test-workspace=true --list

semver-checks: ## Compare the public API against the last published release
# The project promises that a breaking change is deliberate and lands in
# CHANGELOG.md (see CONTRIBUTING.md). This is what checks the first half
# of that promise mechanically instead of by review.
	@$(call ensure,$(SEMVER_CHECKS))
	@cargo semver-checks check-release --workspace

coverage: ## Generate an HTML coverage report (needs cargo-llvm-cov)
	@$(call ensure,cargo-llvm-cov@0.9.0)
	@cargo llvm-cov --workspace --all-features --html
	@echo "Report at target/llvm-cov/html/index.html"

gen-features: ## Regenerate the per-language Cargo features after data/*.sor changes
	@python3 scripts/gen-locale-features.py

sync-data: ## Refresh data/*.sor from upstream libnumbertext (see data/SOURCE.md)
	@bash scripts/sync-data.sh

check-data: ## Verify data/*.sor still matches the commit data/SOURCE.md names
	@python3 scripts/check-data-provenance.py

doc-sync: ## List documentation a change to the public API has left behind
# Deliberately never fails: a diff can touch a public item without any of
# the listed files needing an edit, and a check that cries wolf gets
# ignored.
	@bash scripts/doc-sync-check.sh

msrv: ## Check that the declared MSRV and this Makefile agree
	@declared=$$(grep -m1 '^rust-version' Cargo.toml | cut -d'"' -f2); \
	if [ "$$declared" != "$(MSRV)" ]; then \
		echo "Cargo.toml says rust-version = $$declared, Makefile says MSRV = $(MSRV)"; \
		exit 1; \
	fi; \
	echo "MSRV $$declared agrees in Cargo.toml and Makefile"

msrv-find: ## Find the lowest toolchain that still builds (is the floor too high?)
# `make msrv` checks the declaration is self-consistent and CI's matrix
# checks it actually works. Neither can tell whether the floor is higher
# than it needs to be - only this can, by bisecting toolchains. Slow, and
# advisory: lowering the floor is a deliberate choice, not an automatic
# one, since it widens what this project promises to keep working.
	@$(call ensure,$(CARGO_MSRV))
	@cargo msrv find

check: fmt-check lint msrv spell test test-features ## Everything CI gates a PR on

clean: ## Remove build and test artifacts
	@cargo clean
	@rm -rf mutants.out mutants.out.old
