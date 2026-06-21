# playwright-rs-site

The landing page for `playwright-rs`, served at
[playwright-rust.dev](https://playwright-rust.dev). It is a Leptos CSR
(client-side, compiled to WebAssembly) app built with [Trunk](https://trunkrs.dev)
and styled with Tailwind CSS v4, deployed to GitHub Pages.

It is end-to-end tested by `playwright-rs` itself before every deploy. That
test lives in the sibling crate [`../site-e2e`](../site-e2e) and runs as a gate
in [`.github/workflows/pages.yml`](../../.github/workflows/pages.yml): the site
deploys only if `playwright-rs` can drive the built page and confirm it works
as advertised.

This crate and `site-e2e` are **excluded from the root workspace** (they target
`wasm32` and pull a large dependency tree), so workspace-wide cargo commands and
the publishable crates' supply chain are unaffected. Run cargo commands here
with `--manifest-path`.

## Prerequisites

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

Tailwind is handled automatically: Trunk downloads the pinned Tailwind v4
standalone binary (see `Trunk.toml`), so there is no Node dependency.

## Preview locally

One command — full page (with walkthrough screenshots) and hot reload:

```bash
cargo run -p playwright-rs --features cli -- install chromium   # once
./preview.sh
```

`preview.sh` builds the site, runs the dogfood test once to generate the
walkthrough receipts into `public/receipts/`, then `trunk serve`s with hot
reload. Trunk re-copies `public/receipts/` into `dist/` on every rebuild, so
the screenshots stay visible as you edit. Re-run the script when you want fresh
receipts.

If you only changed components (not the screenshots), plain `trunk serve` is
enough — the last-generated receipts persist:

```bash
trunk serve --open
```

Serves at http://127.0.0.1:8080. `public_url = "/"` (in `Trunk.toml`) means
assets resolve at the server root, so it renders the same way the custom domain
does.

## Build

```bash
cd crates/site
trunk build --release        # outputs static files to crates/site/dist/
```

## Run the dogfood test

The test serves `dist/` and drives it with a real browser, so build first and
make sure Chromium is installed:

```bash
cargo run -p playwright-rs --features cli -- install chromium   # once
cd crates/site && trunk build --release
cargo test --manifest-path crates/site-e2e/Cargo.toml
```

The test skips gracefully if `dist/` is absent, so it never fails a run that
did not build the site.

## Format and lint

The root `cargo fmt --all` / `cargo clippy --workspace` skip this crate (it is
excluded), so check it directly:

```bash
cargo fmt --manifest-path crates/site/Cargo.toml
cargo clippy --manifest-path crates/site/Cargo.toml --target wasm32-unknown-unknown --all-targets -- -D warnings
cargo clippy --manifest-path crates/site-e2e/Cargo.toml --all-targets -- -D warnings
```

## Layout

```
index.html              Trunk entry: build manifest + HTML shell, meta/OG tags
Trunk.toml              build config; pins Tailwind v4; public_url = "/"
input.css               Tailwind v4 entry + the rust/ink theme tokens
build.rs                highlights snippets/ with syntect at build time
snippets/               code samples, one file per language (.rs/.py/.java/.cs/.toml)
public/                 CNAME + crates-io.png, copied verbatim into dist/
src/main.rs             mounts app::App
src/app.rs              composes the page sections
src/components/         Hero, Install, Comparison, CodeTabs, CodeBlock,
                        FeatureCard, Features, icons
src/snippets.rs         includes the build-time-generated highlighted HTML
```

## Syntax highlighting

Snippets are highlighted at build time by `build.rs` using syntect, which is a
build dependency only, so nothing is shipped to the WASM bundle. To add a
snippet:

1. Drop a file in `snippets/`, named by language extension (e.g. `foo.rs`).
2. Reference the generated constant `crate::snippets::FOO_RS` (file stem and
   extension, upper-cased) and render it with `CodeBlock` or `CodeTabs`.

## Notes

- This is CSR (static) because GitHub Pages cannot run a server. All view code
  lives in `#[component]`s, so a future move to SSR (`cargo-leptos`) would
  touch only the entry and build config, not the components.
- The custom domain is set in the repo's Pages settings; `public/CNAME`
  keeps it from resetting on deploys.
