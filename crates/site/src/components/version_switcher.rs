use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Deserialize;

/// Which snapshot this build is: `"dev"` for the main-HEAD build, or the
/// release version (e.g. `"0.14.0"`). Injected by `build.rs` from `SITE_VERSION`.
const CURRENT_VERSION: &str = env!("SITE_VERSION");

/// `/versions.json`, maintained by the deploy: the newest published release and
/// every published version (newest first).
#[derive(Clone, Debug, Default, Deserialize)]
struct Manifest {
    #[serde(default)]
    latest: String,
    #[serde(default)]
    versions: Vec<String>,
}

fn navigate_to(value: &str) {
    let url = match value {
        "dev" => "/dev/".to_string(),
        v => format!("/v{v}/"),
    };
    let _ = window().location().set_href(&url);
}

/// Top bar showing the current docs version, a dropdown to switch between the
/// `dev` (main HEAD) build and every published release, and a banner when the
/// viewer is not on the latest stable version.
#[component]
pub fn VersionSwitcher() -> impl IntoView {
    let manifest = RwSignal::new(None::<Manifest>);

    // Fetch the manifest so a snapshot can list versions released after it was
    // built. Failure (e.g. no manifest yet) leaves just the current version.
    spawn_local(async move {
        if let Ok(resp) = gloo_net::http::Request::get("/versions.json").send().await
            && resp.ok()
            && let Ok(m) = resp.json::<Manifest>().await
        {
            manifest.set(Some(m));
        }
    });

    let is_dev = CURRENT_VERSION == "dev";

    // (message, link label, link href) when the viewer should be nudged elsewhere.
    let banner = move || -> Option<(String, String, String)> {
        let m = manifest.get()?;
        if is_dev {
            let (label, href) = if m.latest.is_empty() {
                ("the latest release".to_string(), "/".to_string())
            } else {
                (format!("v{}", m.latest), format!("/v{}/", m.latest))
            };
            Some((
                "Unreleased dev build (main) — APIs may change.".to_string(),
                format!("Go to {label} →"),
                href,
            ))
        } else if !m.latest.is_empty() && CURRENT_VERSION != m.latest {
            Some((
                format!("You're viewing v{CURRENT_VERSION} — a newer release is available."),
                format!("Go to v{} →", m.latest),
                format!("/v{}/", m.latest),
            ))
        } else {
            None
        }
    };

    view! {
        <div class="w-full border-b border-rust-700/30 bg-ink-800/80 text-sm">
            <div class="mx-auto flex max-w-5xl items-center gap-3 px-6 py-2">
                <label for="version-select" class="text-rust-50/60">"Version"</label>
                <select
                    id="version-select"
                    class="rounded border border-rust-700/40 bg-ink-900 px-2 py-1 text-rust-50"
                    on:change=move |ev| navigate_to(&event_target_value(&ev))
                >
                    <option value="dev" selected=is_dev>"dev (main)"</option>
                    {move || {
                        manifest
                            .get()
                            .map(|m| {
                                m.versions
                                    .iter()
                                    .map(|v| {
                                        let selected = v == CURRENT_VERSION;
                                        view! {
                                            <option value=v.clone() selected=selected>
                                                {format!("v{v}")}
                                            </option>
                                        }
                                    })
                                    .collect_view()
                            })
                    }}
                </select>

                {move || {
                    banner()
                        .map(|(msg, label, href)| {
                            view! {
                                <span class="ml-auto flex items-center gap-2 text-rust-300">
                                    <span>{msg}</span>
                                    <a href=href class="font-semibold underline hover:text-rust-500">
                                        {label}
                                    </a>
                                </span>
                            }
                        })
                }}
            </div>
        </div>
    }
}
