// Playwright driver management
//
// Handles locating the Playwright driver JS package and the Deno runtime
// that executes it. Unlike playwright-python, playwright-java, and
// playwright-dotnet (which run the driver on the Node.js binary bundled
// inside the driver archive), this crate executes the driver's `cli.js`
// with a system-installed Deno (2.x) via Deno's Node compatibility layer.

use crate::{Error, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Arguments prepended to every `deno` invocation that executes the
/// Playwright driver's `cli.js`.
///
/// - `--allow-all`: the driver spawns browsers, binds sockets, and reads or
///   writes arbitrary filesystem paths; a sandboxed driver cannot function.
/// - `--unstable-detect-cjs`: the playwright npm package ships CommonJS
///   without a `"type"` field in its package.json, so Deno needs the flag
///   to load `cli.js` as CommonJS.
/// - `--no-config` / `--no-lock`: never pick up a `deno.json` or
///   `deno.lock` from the caller's working directory; the driver is
///   self-contained.
pub const DENO_RUN_ARGS: &[&str] = &[
    "run",
    "--allow-all",
    "--unstable-detect-cjs",
    "--no-config",
    "--no-lock",
];

/// Get the paths needed to run the Playwright driver.
///
/// The driver `cli.js` is located in the following order:
/// 1. Bundled driver downloaded by build.rs (PRIMARY)
/// 2. User cache populated by `playwright-rs install` (stable across cargo install)
/// 3. PLAYWRIGHT_DRIVER_PATH environment variable (user override)
/// 4. PLAYWRIGHT_CLI_JS environment variable (user override)
/// 5. Global npm installation (`npm root -g`) (development fallback)
/// 6. Local npm installation (`npm root`) (development fallback)
///
/// The Deno executable is resolved from PATH, `$DENO_INSTALL/bin`,
/// `$HOME/.deno/bin`, and common install locations.
///
/// Returns a tuple of (deno_executable_path, cli_js_path).
///
/// # Errors
///
/// Returns `Error::ServerNotFound` if the driver cannot be located in any
/// of the search paths, and `Error::LaunchFailed` if Deno is not installed.
///
/// # Example
///
/// ```no_run
/// use playwright_rs::server::driver::get_driver_executable;
///
/// let (deno_exe, cli_js) = get_driver_executable()?;
/// println!("Deno: {}", deno_exe.display());
/// println!("CLI:  {}", cli_js.display());
/// # Ok::<(), playwright_rs::Error>(())
/// ```
pub fn get_driver_executable() -> Result<(PathBuf, PathBuf)> {
    let cli_js = try_bundled_cli()
        .or_else(try_user_cache_cli)
        .or_else(try_driver_path_env)
        .or_else(try_cli_js_env)
        .or_else(try_npm_global)
        .or_else(try_npm_local)
        .ok_or(Error::ServerNotFound)?;

    let deno_exe = find_deno_executable()?;
    Ok((deno_exe, cli_js))
}

/// Try to find the bundled driver's cli.js from build.rs
///
/// This is the PRIMARY path: build.rs downloads the driver archive and
/// records its location at compile time.
fn try_bundled_cli() -> Option<PathBuf> {
    if let Some(cli_js) = option_env!("PLAYWRIGHT_CLI_JS") {
        let cli_path = PathBuf::from(cli_js);
        if cli_path.exists() {
            return Some(cli_path);
        }
    }

    if let Some(driver_dir) = option_env!("PLAYWRIGHT_DRIVER_DIR") {
        let cli_js = PathBuf::from(driver_dir).join("package").join("cli.js");
        if cli_js.exists() {
            return Some(cli_js);
        }
    }

    None
}

/// Try to find the driver in the user cache populated by `playwright-rs install`.
///
/// The CLI bootstrap drops the driver at
/// `<cache>/playwright-rust/<version>/playwright-<version>-<platform>/`,
/// which survives `cargo install` cleanup of the build's `target/`. The
/// version and platform come from compile-time env vars emitted by build.rs.
fn try_user_cache_cli() -> Option<PathBuf> {
    let cache_dir = dirs::cache_dir()?;
    let version = option_env!("PLAYWRIGHT_DRIVER_VERSION")?;
    let platform = option_env!("PLAYWRIGHT_DRIVER_PLATFORM")?;
    try_user_cache_cli_in(&cache_dir, version, platform)
}

/// Resolution helper for `try_user_cache_cli` parameterised by cache root,
/// version, and platform — exposed at module scope so tests can drive it
/// with a `tempfile::tempdir()`.
fn try_user_cache_cli_in(cache_root: &Path, version: &str, platform: &str) -> Option<PathBuf> {
    let cli_js = cache_root
        .join("playwright-rust")
        .join(version)
        .join(format!("playwright-{}-{}", version, platform))
        .join("package")
        .join("cli.js");

    cli_js.exists().then_some(cli_js)
}

/// Try to find the driver from the PLAYWRIGHT_DRIVER_PATH environment variable
///
/// User can set PLAYWRIGHT_DRIVER_PATH to a directory containing `package/cli.js`.
fn try_driver_path_env() -> Option<PathBuf> {
    let driver_path = std::env::var("PLAYWRIGHT_DRIVER_PATH").ok()?;
    let cli_js = PathBuf::from(driver_path).join("package").join("cli.js");
    cli_js.exists().then_some(cli_js)
}

/// Try to find the driver from the PLAYWRIGHT_CLI_JS environment variable
///
/// User can set the variable to explicitly specify the cli.js path.
fn try_cli_js_env() -> Option<PathBuf> {
    let cli_js = PathBuf::from(std::env::var("PLAYWRIGHT_CLI_JS").ok()?);
    cli_js.exists().then_some(cli_js)
}

/// Try to find the driver in an npm global installation (development fallback)
fn try_npm_global() -> Option<PathBuf> {
    let output = Command::new("npm").args(["root", "-g"]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let npm_root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    find_playwright_in_node_modules(&PathBuf::from(npm_root))
}

/// Try to find the driver in a local npm installation (development fallback)
fn try_npm_local() -> Option<PathBuf> {
    let output = Command::new("npm").args(["root"]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let npm_root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    find_playwright_in_node_modules(&PathBuf::from(npm_root))
}

/// Find the Playwright cli.js in a node_modules directory
fn find_playwright_in_node_modules(node_modules: &Path) -> Option<PathBuf> {
    let playwright_dirs = [
        node_modules.join("playwright"),
        node_modules.join("@playwright").join("test"),
    ];

    for playwright_dir in &playwright_dirs {
        let cli_js = playwright_dir.join("cli.js");
        if cli_js.exists() {
            return Some(cli_js);
        }
    }

    None
}

/// Find the Deno executable in PATH or common install locations
fn find_deno_executable() -> Result<PathBuf> {
    #[cfg(not(windows))]
    let which_cmd = "which";
    #[cfg(windows)]
    let which_cmd = "where";

    if let Ok(output) = Command::new(which_cmd).arg("deno").output()
        && output.status.success()
    {
        let deno_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !deno_path.is_empty() {
            let path = PathBuf::from(deno_path.lines().next().unwrap_or(&deno_path));
            if path.exists() {
                return Ok(path);
            }
        }
    }

    let deno_bin = if cfg!(windows) { "deno.exe" } else { "deno" };

    if let Ok(install_root) = std::env::var("DENO_INSTALL") {
        let path = PathBuf::from(install_root).join("bin").join(deno_bin);
        if path.exists() {
            return Ok(path);
        }
    }

    if let Some(home) = dirs::home_dir() {
        let path = home.join(".deno").join("bin").join(deno_bin);
        if path.exists() {
            return Ok(path);
        }
    }

    #[cfg(not(windows))]
    let common_locations = [
        "/usr/local/bin/deno",
        "/usr/bin/deno",
        "/opt/homebrew/bin/deno",
        "/opt/local/bin/deno",
    ];

    #[cfg(windows)]
    let common_locations = [
        "C:\\Program Files\\deno\\deno.exe",
        "C:\\ProgramData\\chocolatey\\bin\\deno.exe",
    ];

    for location in &common_locations {
        let path = PathBuf::from(location);
        if path.exists() {
            return Ok(path);
        }
    }

    Err(Error::LaunchFailed(
        "Deno executable not found. Install Deno 2.x (https://deno.com) or add it to PATH."
            .to_string(),
    ))
}

/// Install Playwright browsers programmatically.
///
/// Finds the Playwright driver and runs:
/// `deno run --allow-all <driver>/package/cli.js install [browsers...]`
///
/// # Parameters
///
/// - `browsers` — optional slice of browser names (e.g. `&["chromium", "firefox"]`).
///   Pass `None` to install all browsers (equivalent to `npx playwright install`).
///   Pass `Some(&[])` for a no-op invocation that validates the driver is reachable.
///
/// On Linux, `--with-deps` is automatically appended so that required system
/// libraries (libgtk, libnss, etc.) are installed alongside the browser binaries.
/// Use [`install_browsers_with_deps`] to force this flag on other platforms.
///
/// # Errors
///
/// - [`Error::ServerNotFound`] if the Playwright driver cannot be located.
/// - [`Error::LaunchFailed`] if Deno is not installed, or the installation
///   process exits with a non-zero status or fails to spawn.
///
/// # Example
///
/// ```no_run
/// use playwright_rs::install_browsers;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Install only Chromium
///     install_browsers(Some(&["chromium"])).await?;
///
///     // Install all browsers
///     install_browsers(None).await?;
///     Ok(())
/// }
/// ```
///
/// See: <https://playwright.dev/docs/browsers#installing-browsers>
pub async fn install_browsers(browsers: Option<&[&str]>) -> Result<()> {
    install_browsers_impl(browsers, /* with_deps_forced */ false).await
}

/// Install Playwright browsers and their system dependencies.
///
/// Identical to [`install_browsers`] but always passes `--with-deps` to the
/// Playwright CLI, regardless of the current operating system. This is the
/// recommended call for CI environments where system libraries may be missing.
///
/// # Parameters
///
/// - `browsers` — optional slice of browser names. `None` installs all browsers.
///
/// # Errors
///
/// - [`Error::ServerNotFound`] if the Playwright driver cannot be located.
/// - [`Error::LaunchFailed`] if Deno is not installed, or the installation
///   process exits with a non-zero status or fails to spawn.
///
/// # Example
///
/// ```no_run
/// use playwright_rs::install_browsers_with_deps;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     install_browsers_with_deps(Some(&["chromium", "firefox"])).await?;
///     Ok(())
/// }
/// ```
///
/// See: <https://playwright.dev/docs/browsers#installing-browsers>
pub async fn install_browsers_with_deps(browsers: Option<&[&str]>) -> Result<()> {
    install_browsers_impl(browsers, /* with_deps_forced */ true).await
}

/// Internal implementation shared by [`install_browsers`] and [`install_browsers_with_deps`].
async fn install_browsers_impl(browsers: Option<&[&str]>, with_deps_forced: bool) -> Result<()> {
    let (deno_exe, cli_js) = get_driver_executable()?;

    let mut cmd = tokio::process::Command::new(&deno_exe);
    cmd.args(DENO_RUN_ARGS).arg(&cli_js).arg("install");

    if let Some(browser_list) = browsers {
        for browser in browser_list {
            cmd.arg(browser);
        }
    }

    // Pass --with-deps on Linux automatically (needed for system libraries),
    // or when the caller explicitly requested it via install_browsers_with_deps.
    if with_deps_forced || cfg!(target_os = "linux") {
        cmd.arg("--with-deps");
    }

    let output = cmd.output().await.map_err(|e| {
        Error::LaunchFailed(format!("Failed to spawn browser install process: {}", e))
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(Error::LaunchFailed(format!(
            "Browser installation failed (exit code {:?}).\nstdout: {}\nstderr: {}",
            output.status.code(),
            stdout.trim(),
            stderr.trim(),
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_deno_executable() {
        // This should succeed on any system with Deno installed
        let result = find_deno_executable();
        match result {
            Ok(deno_path) => {
                tracing::info!("Found deno at: {:?}", deno_path);
                assert!(deno_path.exists());
            }
            Err(e) => {
                tracing::warn!("Deno not found (expected if Deno not installed): {:?}", e);
                // Don't fail the test if Deno is not installed
            }
        }
    }

    #[test]
    fn test_get_driver_executable() {
        // This test will pass if any driver source is available
        let result = get_driver_executable();
        match result {
            Ok((deno, cli)) => {
                tracing::info!("Found Playwright driver:");
                tracing::info!("  Deno: {:?}", deno);
                tracing::info!("  CLI:  {:?}", cli);
                assert!(deno.exists());
                assert!(cli.exists());
            }
            Err(Error::ServerNotFound) => {
                tracing::warn!("Playwright driver not found (expected in some environments)");
                tracing::warn!(
                    "This is OK - driver will be bundled at build time or can be installed via npm"
                );
            }
            Err(Error::LaunchFailed(msg)) => {
                tracing::warn!("Deno not found (expected in some environments): {msg}");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_bundled_cli_detection() {
        // Test that we can detect the bundled driver if build.rs set env vars
        let result = try_bundled_cli();
        match result {
            Some(cli) => {
                tracing::info!("Found bundled driver cli.js: {:?}", cli);
                assert!(cli.exists());
            }
            None => {
                tracing::info!("No bundled driver (expected during development)");
            }
        }
    }

    #[test]
    fn try_user_cache_cli_in_resolves_when_files_present() {
        let temp = tempfile::tempdir().unwrap();
        let driver_subdir = temp
            .path()
            .join("playwright-rust")
            .join("1.60.0")
            .join("playwright-1.60.0-linux");
        std::fs::create_dir_all(driver_subdir.join("package")).unwrap();
        std::fs::write(driver_subdir.join("package").join("cli.js"), b"").unwrap();

        let cli = try_user_cache_cli_in(temp.path(), "1.60.0", "linux").unwrap();
        assert!(cli.exists());
    }

    #[test]
    fn try_user_cache_cli_in_returns_none_when_absent() {
        let temp = tempfile::tempdir().unwrap();
        let result = try_user_cache_cli_in(temp.path(), "1.60.0", "linux");
        assert!(result.is_none());
    }

    #[test]
    fn bundled_driver_dir_lives_under_out_dir() {
        // Only meaningful for the default download location. CI relocates the
        // driver via PLAYWRIGHT_DRIVER_CACHE_DIR (cached on its own key) and
        // compile-only jobs skip the download entirely; in those modes the
        // OUT_DIR layout intentionally does not apply.
        if env!("PLAYWRIGHT_DRIVER_DIR_SOURCE") != "out_dir" {
            return;
        }
        let dir = env!("PLAYWRIGHT_DRIVER_DIR");
        let sep = std::path::MAIN_SEPARATOR;
        let build_marker = format!("{sep}build{sep}playwright-rs");
        let out_marker = format!("{sep}out{sep}");
        assert!(
            dir.contains(&build_marker) && dir.contains(&out_marker),
            "PLAYWRIGHT_DRIVER_DIR should sit under target/<profile>/build/playwright-rs-<hash>/out, got: {dir}"
        );
    }
}
