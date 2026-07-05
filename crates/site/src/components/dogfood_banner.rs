use leptos::prelude::*;

const E2E_TEST: &str =
    "https://github.com/padamson/playwright-rust/blob/main/crates/site-e2e/tests/landing_page.rs";
const TRACE_VIEWER: &str = "https://trace.playwright.dev";
// Receipts written by the dogfood test into dist/receipts/ on each deploy.
const TRACE: &str = "receipts/trace.zip";
const HAR: &str = "receipts/dogfood.har";
const ARIA: &str = "receipts/aria-snapshot.txt";
const SCREENCAST: &str = "receipts/screencast.jpeg";

#[component]
pub fn DogfoodBanner() -> impl IntoView {
    let card = "flex flex-col rounded-xl border border-rust-700/40 bg-ink-800/60 p-5 text-left";
    let dl = "mt-3 text-sm font-semibold text-rust-300 underline hover:text-rust-500";

    view! {
        <section id="dogfood-banner" class="mx-auto max-w-5xl px-6 py-12">
            <div class="rounded-2xl border border-rust-500/40 bg-rust-500/5 p-8 text-center">
                <h2 class="text-2xl font-bold text-rust-300">
                    "Tested by the binding it advertises"
                </h2>
                <p class="mx-auto mt-3 max-w-2xl text-rust-50/80">
                    "This page is a Leptos app built in Rust, and playwright-rs drives it end to "
                    "end in CI; it deploys only if the binding confirms it works. Every run leaves "
                    "these receipts:"
                </p>

                <div class="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-3">
                    <div class=card>
                        <h3 class="font-semibold text-rust-300">"Trace"</h3>
                        <p class="mt-1 flex-1 text-sm text-rust-50/70">
                            "Step-by-step timeline of the run. Open it in "
                            <a href=TRACE_VIEWER class="underline hover:text-rust-300">
                                "trace.playwright.dev"
                            </a> "."
                        </p>
                        <a href=TRACE download class=dl>"Download .zip"</a>
                    </div>
                    <div class=card>
                        <h3 class="font-semibold text-rust-300">"Network HAR"</h3>
                        <p class="mt-1 flex-1 text-sm text-rust-50/70">
                            "Every request the run made. Open in browser devtools or replay it "
                            "with playwright-rs."
                        </p>
                        <a href=HAR download class=dl>"Download .har"</a>
                    </div>
                    <div class=card>
                        <h3 class="font-semibold text-rust-300">"Accessibility tree"</h3>
                        <p class="mt-1 flex-1 text-sm text-rust-50/70">
                            "The ARIA snapshot the test asserts, so the page's semantic structure "
                            "can't regress unnoticed."
                        </p>
                        <a href=ARIA download class=dl>"Download .txt"</a>
                    </div>
                    // Dev-only: a frame the run captured via the unreleased 1.61.0
                    // screencast API (with cursor decoration). Omitted on release
                    // snapshots until the feature ships.
                    {(env!("SITE_VERSION") == "dev").then(|| view! {
                        <div class=card>
                            <div class="flex items-center gap-2">
                                <h3 class="font-semibold text-rust-300">"Screencast frame"</h3>
                                <span class="rounded-full border border-rust-500/50 bg-rust-500/10 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wider text-rust-300">
                                    "Unreleased"
                                </span>
                            </div>
                            <p class="mt-1 flex-1 text-sm text-rust-50/70">
                                "A frame is captured with the new screencast API and "
                                "cursor decoration."
                            </p>
                            <a href=SCREENCAST download class=dl>"Download .jpeg"</a>
                        </div>
                    })}
                </div>

                <p class="mt-6 text-sm font-semibold">
                    <a href=E2E_TEST class="text-rust-300 underline hover:text-rust-500">
                        "See the test that produces these"
                    </a>
                </p>
            </div>
        </section>
    }
}
