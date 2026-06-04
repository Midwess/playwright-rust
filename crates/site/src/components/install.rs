use leptos::prelude::*;

use super::CodeBlock;
use crate::snippets;

#[component]
pub fn Install() -> impl IntoView {
    view! {
        <section id="install" class="mx-auto max-w-3xl px-6 py-12">
            <h2 class="mb-4 text-2xl font-bold text-rust-300">"Install"</h2>
            <CodeBlock html=snippets::INSTALL_TOML/>
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
