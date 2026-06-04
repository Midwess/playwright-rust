use leptos::prelude::*;

const CRATES_IO: &str = "https://crates.io/crates/playwright-rs";
const DOCS_RS: &str = "https://docs.rs/playwright-rs";
const GITHUB: &str = "https://github.com/padamson/playwright-rust";
const PLAYWRIGHT_DEV: &str = "https://playwright.dev";

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer id="footer" class="mt-8 border-t border-rust-700/30 px-6 py-10">
            <div class="mx-auto flex max-w-5xl flex-col gap-4 text-sm text-rust-50/60">
                <nav class="flex flex-wrap gap-5">
                    <a href=GITHUB class="hover:text-rust-300">"GitHub"</a>
                    <a href=DOCS_RS class="hover:text-rust-300">"Docs"</a>
                    <a href=CRATES_IO class="hover:text-rust-300">"crates.io"</a>
                </nav>
                <p id="disclaimer" class="max-w-3xl">
                    "playwright-rs is an unofficial, community-maintained project. It is not "
                    "affiliated with or endorsed by Microsoft. \"Playwright\" is a trademark of "
                    "Microsoft, and the official bindings live at "
                    <a href=PLAYWRIGHT_DEV class="underline hover:text-rust-300">"playwright.dev"</a>
                    "."
                </p>
                <p>"Licensed under Apache-2.0. Built with Leptos and Trunk."</p>
            </div>
        </footer>
    }
}
