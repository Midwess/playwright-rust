#!/usr/bin/env bash
#
# Run cargo-mutants locally over the lines changed in HEAD (or any base
# ref you choose), so the runtime is in the "useful while iterating"
# range rather than "leave on overnight."
#
# Usage:
#   ./scripts/mutants.sh                      # diff HEAD~1..HEAD
#   ./scripts/mutants.sh main                 # diff main..HEAD
#   ./scripts/mutants.sh 0bb7329              # diff <sha>..HEAD
#   ./scripts/mutants.sh HEAD~5               # diff last 5 commits
#   ./scripts/mutants.sh -- --jobs 4          # default base + extra cargo-mutants args
#   ./scripts/mutants.sh main --jobs 4        # explicit base + extra args
#
# The first non-dash argument is the base ref; anything else (and
# everything after the first dash-prefixed arg) passes through to
# cargo-mutants. See https://mutants.rs/ for the full CLI surface.
#
# Why `--in-diff`: an unscoped `cargo mutants` run grows linearly with
# codebase size and routinely runs many hours. `--in-diff` narrows
# mutation to just the lines in the supplied diff — typically seconds
# to minutes for a normal commit.
#
# Scope: `.cargo/mutants.toml` lists the files we mutate at all
# (`examine_globs`) plus mutants we exclude as integration-only. The
# `--in-diff` filter intersects with that scope — diffs touching files
# outside `examine_globs` produce no mutants, which is intentional.
#
# Prerequisites: `cargo install cargo-mutants` (once per machine).
#
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

# Mutation testing runs lib + serialization tests only (see
# .cargo/mutants.toml) — never a browser — so skip the ~42 MB Playwright
# driver download build.rs would otherwise do in every scratch build.
export PLAYWRIGHT_SKIP_DRIVER_DOWNLOAD=1

# Incremental is disabled globally (.cargo/config.toml) because it bloated
# target/ with no payoff for normal builds, but cargo-mutants rebuilds once
# per mutant and genuinely benefits from it — re-enable for mutation runs.
export CARGO_INCREMENTAL=1

# --- Project-specific pre-setup ----------------------------------------
# cargo-mutants copies the source tree to a tempdir before building.
# Anything the build expects but that isn't checked in (wasm-pack
# artifacts referenced via include_str!/include_bytes!, generated FFI
# bindings, etc.) won't follow. Mutation testing doesn't exercise those
# bytes at runtime, so empty placeholders are enough.
#
# playwright-rust currently has no such artifacts (the Playwright driver
# is skipped above). Add placeholders here if a future change starts
# include!()-ing checked-out-only files.
# -----------------------------------------------------------------------

# Resolve the base ref: first non-dash positional arg, defaulting to
# HEAD~1. Anything starting with `-` is treated as a cargo-mutants arg.
if [[ $# -gt 0 && "$1" != -* && "$1" != "--" ]]; then
  BASE="$1"
  shift
else
  BASE="HEAD~1"
fi

# `--` separator is allowed for clarity; consume it so it doesn't pass
# through to cargo-mutants as a positional.
if [[ $# -gt 0 && "$1" == "--" ]]; then
  shift
fi

DIFF="$(mktemp -t mutants.XXXXXX.diff)"
trap 'rm -f "$DIFF"' EXIT

git diff "${BASE}..HEAD" > "$DIFF"

if [[ ! -s "$DIFF" ]]; then
  echo "no diff between ${BASE} and HEAD — nothing to mutate."
  echo "tip: commit your changes locally first, then re-run."
  exit 0
fi

echo "mutating changes in ${BASE}..HEAD ($(wc -l < "$DIFF") diff lines)"
exec cargo mutants --in-diff "$DIFF" "$@"
