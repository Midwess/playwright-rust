use leptos::prelude::*;

use super::CodeBlock;
use crate::snippets;

#[component]
pub fn Install() -> impl IntoView {
    // The dev (main HEAD) build is unreleased, so it installs from git, not the
    // crates.io version — that's how you get the features previewed below.
    let is_dev = env!("SITE_VERSION") == "dev";

    view! {
        <section id="install" class="mx-auto max-w-3xl px-6 py-12">
            <h2 class="mb-4 text-2xl font-bold text-rust-300">"Install"</h2>
            {if is_dev {
                view! {
                    <CodeBlock
                        html=snippets::INSTALL_DEV_TOML
                        caption="Unreleased — installs from GitHub main HEAD"
                    />
                }
                    .into_any()
            } else {
                view! { <CodeBlock html=snippets::INSTALL_TOML/> }.into_any()
            }}
            <p class="mt-4 text-sm text-rust-50/70">
                "The default "
                <code class="text-rust-300">"macros"</code>
                " feature ships the compile-time "
                <code class="text-rust-300">"locator!()"</code>
                " selector macro. Turn on "
                <code class="text-rust-300">"cli"</code>
                " for the browser-installer binary, or "
                <code class="text-rust-300">"screenshot-diff"</code>
                " for pixel-diff assertions."
            </p>
        </section>
    }
}
