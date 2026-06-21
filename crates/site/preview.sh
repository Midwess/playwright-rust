#!/usr/bin/env bash
#
# Local preview with walkthrough screenshots and hot reload.
#
# Builds the site, runs the dogfood test once to (re)generate the receipts into
# public/receipts/, then serves with Trunk. Trunk re-copies public/receipts/
# into dist on every rebuild, so the screenshots stay visible through hot
# reload. Edit components and the page updates live; re-run this script when you
# want fresh receipts.
#
# Prereq (once): cargo run -p playwright-rs --features cli -- install chromium
set -euo pipefail

cd "$(dirname "$0")"

trunk build
cargo test --manifest-path ../site-e2e/Cargo.toml
exec trunk serve --open
