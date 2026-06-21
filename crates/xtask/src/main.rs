//! Workspace build-tooling binary. See `cargo xtask --help`.
//!
//! Subcommands:
//! - `regenerate-trace-fixture` — drives a real Chromium session via
//!   `playwright-rs` and writes the resulting `.trace.zip` into
//!   `crates/playwright-rs-trace/tests/fixtures/`, which downstream
//!   parser tests consume.
//! - `verify-agent-docs` — extracts ` ```rust,no_run ` code blocks
//!   from `docs/agent/CLAUDE_SNIPPET.md` and
//!   `.claude/skills/playwright-rs-usage/SKILL.md`, then runs
//!   `cargo check` against them so they can't silently drift from the
//!   crate API.
//! - `verify-site-snippets` — wraps each `crates/site/snippets/*.rs`
//!   fragment shown on the landing page in a binding prelude and runs
//!   `cargo check`, so the site can't advertise code that doesn't
//!   compile.

use anyhow::{Context as _, Result, bail};
use axum::Router;
use axum::routing::get;
use clap::Parser;
use playwright_rs::Playwright;
use playwright_rs::protocol::{TracingStartOptions, TracingStopOptions};
use std::path::{Path, PathBuf};

/// `cargo xtask <subcommand>`
#[derive(Parser)]
#[command(name = "xtask", about = "Workspace build tooling for playwright-rust")]
enum Cmd {
    /// Regenerate the deterministic trace fixture used by
    /// playwright-rs-trace's parse tests.
    RegenerateTraceFixture {
        /// Output zip path (defaults to the fixture location).
        #[arg(
            long,
            default_value = "crates/playwright-rs-trace/tests/fixtures/basic.trace.zip"
        )]
        out: PathBuf,
    },
    /// Compile-check the `rust,no_run` blocks in the agent-integration
    /// docs (`docs/agent/CLAUDE_SNIPPET.md` and
    /// `.claude/skills/playwright-rs-usage/SKILL.md`) so they can't
    /// drift from the real `playwright-rs` API.
    VerifyAgentDocs,
    /// Compile-check the Rust snippets rendered on the landing page
    /// (`crates/site/snippets/*.rs`) so the site can't advertise code
    /// that doesn't compile against the real `playwright-rs` API.
    VerifySiteSnippets,
}

#[tokio::main]
async fn main() -> Result<()> {
    match Cmd::parse() {
        Cmd::RegenerateTraceFixture { out } => regenerate_trace_fixture(&out).await,
        Cmd::VerifyAgentDocs => verify_agent_docs(),
        Cmd::VerifySiteSnippets => verify_site_snippets(),
    }
}

async fn regenerate_trace_fixture(out: &Path) -> Result<()> {
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create parent dir {}", parent.display()))?;
    }

    // Local server so the navigation produces a `resource-snapshot`
    // in `trace.network` — `data:` URLs don't.
    let app = Router::new().route(
        "/",
        get(|| async { axum::response::Html(FIXTURE_PAGE_HTML) }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .context("bind fixture server")?;
    let addr = listener.local_addr().context("local_addr")?;
    let server = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    let pw = Playwright::launch()
        .await
        .context("launch playwright server")?;
    let browser = pw.chromium().launch().await.context("launch chromium")?;
    let context = browser.new_context().await.context("new browser context")?;
    let tracing = context.tracing().await.context("get tracing handle")?;

    tracing
        .start(Some(
            TracingStartOptions::default()
                .name("fixture")
                .screenshots(true)
                .snapshots(true),
        ))
        .await
        .context("start tracing")?;

    let page = context.new_page().await.context("new page")?;

    page.goto(&format!("http://{addr}/"), None)
        .await
        .context("goto local fixture server")?;

    page.locator("#b")
        .await
        .click(None)
        .await
        .context("click button")?;

    // Brief pause so the console event is recorded before stop.
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let out_str = out.to_string_lossy().into_owned();
    tracing
        .stop(Some(TracingStopOptions::default().path(out_str.clone())))
        .await
        .context("stop tracing")?;

    browser.close().await.context("close browser")?;
    server.abort();

    println!("wrote {}", out.display());
    Ok(())
}

const FIXTURE_PAGE_HTML: &str = r#"<!doctype html>
<html>
<body>
<button id="b" onclick='console.log("hi")'>X</button>
</body>
</html>"#;

/// Walk the agent-doc markdown files, extract every ` ```rust,no_run `
/// block, write them into a throwaway crate under
/// `target/agent-docs-verify/`, and run `cargo check` against it. If
/// the snippets drift from the real `playwright-rs` API, this fails.
fn verify_agent_docs() -> Result<()> {
    let workspace_root = workspace_root();
    let inputs = [
        workspace_root.join("docs/agent/CLAUDE_SNIPPET.md"),
        workspace_root.join(".claude/skills/playwright-rs-usage/SKILL.md"),
    ];

    let mut blocks: Vec<ExtractedBlock> = Vec::new();
    for path in &inputs {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        blocks.extend(extract_no_run_blocks(path, &content));
    }

    if blocks.is_empty() {
        println!("verify-agent-docs: no `rust,no_run` blocks found — nothing to check");
        return Ok(());
    }

    let check_dir = workspace_root.join("target/agent-docs-verify");
    std::fs::create_dir_all(check_dir.join("tests"))
        .with_context(|| format!("create {}", check_dir.display()))?;

    let playwright_path = workspace_root.join("crates/playwright");
    let cargo_toml = format!(
        r#"# Generated by `cargo xtask verify-agent-docs`. Do not edit.
[workspace]

[package]
name = "agent-docs-verify"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
anyhow = "1"
tokio = {{ version = "1", features = ["macros", "rt-multi-thread"] }}
playwright-rs = {{ path = {playwright_path:?} }}
"#,
        playwright_path = playwright_path.display().to_string(),
    );
    std::fs::write(check_dir.join("Cargo.toml"), cargo_toml).context("write Cargo.toml")?;

    let mut tests_rs = String::from(
        "// Generated by `cargo xtask verify-agent-docs`. Do not edit.\n\
         #![allow(unused_imports, unused_variables, dead_code)]\n\n",
    );
    for block in &blocks {
        let rel = block
            .source
            .strip_prefix(&workspace_root)
            .unwrap_or(&block.source);
        tests_rs.push_str(&format!(
            "// === {} (line {}) ===\n",
            rel.display(),
            block.start_line
        ));
        tests_rs.push_str(&block.code);
        tests_rs.push_str("\n\n");
    }
    std::fs::write(check_dir.join("tests/agent_docs.rs"), tests_rs)
        .context("write tests/agent_docs.rs")?;

    let status = std::process::Command::new(env!("CARGO"))
        .args(["check", "--tests", "--manifest-path"])
        .arg(check_dir.join("Cargo.toml"))
        .status()
        .context("invoke cargo check")?;

    if !status.success() {
        bail!(
            "verify-agent-docs: cargo check failed — agent-doc snippets have drifted from the playwright-rs API. \
             See output above; offending source lines are noted in `target/agent-docs-verify/tests/agent_docs.rs`."
        );
    }

    println!(
        "verify-agent-docs: {} block(s) compile cleanly against playwright-rs",
        blocks.len()
    );
    Ok(())
}

fn verify_site_snippets() -> Result<()> {
    let workspace_root = workspace_root();
    let snippets_dir = workspace_root.join("crates/site/snippets");

    let mut paths: Vec<PathBuf> = std::fs::read_dir(&snippets_dir)
        .with_context(|| format!("read {}", snippets_dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|ext| ext == "rs"))
        .collect();
    paths.sort();

    if paths.is_empty() {
        println!("verify-site-snippets: no .rs snippets found — nothing to check");
        return Ok(());
    }

    let check_dir = workspace_root.join("target/site-snippets-verify");
    std::fs::create_dir_all(check_dir.join("tests"))
        .with_context(|| format!("create {}", check_dir.display()))?;

    let playwright_path = workspace_root.join("crates/playwright");
    let cargo_toml = format!(
        r#"# Generated by `cargo xtask verify-site-snippets`. Do not edit.
[workspace]

[package]
name = "site-snippets-verify"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
tokio = {{ version = "1", features = ["macros", "rt-multi-thread"] }}
tracing-subscriber = "0.3"
playwright-rs = {{ path = {playwright_path:?} }}
"#,
        playwright_path = playwright_path.display().to_string(),
    );
    std::fs::write(check_dir.join("Cargo.toml"), cargo_toml).context("write Cargo.toml")?;

    // Each snippet is a fragment shown on the landing page, not a full
    // program. Wrap it in an async fn whose prelude binds everything the
    // fragments assume to be in scope (pw / page / url / context / cards).
    let mut tests_rs = String::from(
        "// Generated by `cargo xtask verify-site-snippets`. Do not edit.\n\
         #![allow(unused_imports, unused_variables, unused_mut, dead_code)]\n\
         use playwright_rs::protocol::{Playwright, ScreenshotOptions, Viewport};\n\
         use playwright_rs::expect;\n\n",
    );
    for path in &paths {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("snippet")
            .replace('-', "_");
        let code =
            std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        let rel = path.strip_prefix(&workspace_root).unwrap_or(path);
        tests_rs.push_str(&format!(
            "// === {} ===\n\
             async fn snippet_{stem}() -> std::result::Result<(), Box<dyn std::error::Error>> {{\n\
             \x20   let pw = Playwright::launch().await?;\n\
             \x20   let browser = pw.chromium().launch().await?;\n\
             \x20   let context = browser.new_context().await?;\n\
             \x20   let page = context.new_page().await?;\n\
             \x20   let url = \"http://localhost:8080\";\n\
             \x20   let cards: Vec<(&str, &str)> = Vec::new();\n\
             \x20   {{\n",
            rel.display(),
        ));
        for line in code.lines() {
            tests_rs.push_str("        ");
            tests_rs.push_str(line);
            tests_rs.push('\n');
        }
        tests_rs.push_str("    }\n    Ok(())\n}\n\n");
    }
    std::fs::write(check_dir.join("tests/snippets.rs"), tests_rs)
        .context("write tests/snippets.rs")?;

    let status = std::process::Command::new(env!("CARGO"))
        .args(["check", "--tests", "--manifest-path"])
        .arg(check_dir.join("Cargo.toml"))
        // Compile-only: never launches a browser, so skip the ~42 MB
        // driver download in playwright-rs's build script.
        .env("PLAYWRIGHT_SKIP_DRIVER_DOWNLOAD", "1")
        .status()
        .context("invoke cargo check")?;

    if !status.success() {
        bail!(
            "verify-site-snippets: cargo check failed — a landing-page snippet has drifted from the \
             playwright-rs API. See output above; sources are noted in \
             `target/site-snippets-verify/tests/snippets.rs`."
        );
    }

    println!(
        "verify-site-snippets: {} snippet(s) compile cleanly against playwright-rs",
        paths.len()
    );
    Ok(())
}

struct ExtractedBlock {
    source: PathBuf,
    start_line: usize,
    code: String,
}

/// Parse fenced markdown code blocks tagged ` ```rust,no_run `. The
/// closing fence (``` ``` ```) must match the opening; no nesting.
fn extract_no_run_blocks(path: &Path, content: &str) -> Vec<ExtractedBlock> {
    let mut blocks = Vec::new();
    let mut in_block = false;
    let mut start_line = 0usize;
    let mut buf = String::new();

    for (idx, line) in content.lines().enumerate() {
        let lineno = idx + 1;
        let trimmed = line.trim();
        if !in_block {
            // Match `rust,no_run` with optional trailing whitespace.
            if trimmed == "```rust,no_run" {
                in_block = true;
                start_line = lineno + 1;
                buf.clear();
            }
        } else if trimmed == "```" {
            blocks.push(ExtractedBlock {
                source: path.to_path_buf(),
                start_line,
                code: buf.clone(),
            });
            in_block = false;
        } else {
            buf.push_str(line);
            buf.push('\n');
        }
    }

    blocks
}

/// Resolve the workspace root by walking up from the xtask binary's
/// `CARGO_MANIFEST_DIR` (which Cargo sets at compile time for the
/// xtask crate to `crates/xtask`).
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("xtask manifest dir has two parents")
        .to_path_buf()
}
