use leptos::prelude::*;

use super::{CodeBlock, CodeTabs};
use crate::snippets;

#[component]
pub fn Comparison() -> impl IntoView {
    view! {
        <section id="comparison" class="mx-auto max-w-5xl px-6 py-12">
            <h2 class="mb-2 text-2xl font-bold text-rust-300">"Familiar from day one"</h2>
            <p class="mb-6 text-sm text-rust-50/70">
                "The API follows Playwright's cross-language conventions. If you know "
                "playwright-python, playwright-java, or playwright-dotnet, you already know "
                "playwright-rs. Pick the language you know on the left and compare it with the "
                "Rust on the right."
            </p>
            <div class="grid grid-cols-1 gap-6 md:grid-cols-2">
                <CodeTabs tabs=vec![
                    ("Python", snippets::EXAMPLE_PY),
                    ("Java", snippets::EXAMPLE_JAVA),
                    (".NET", snippets::EXAMPLE_CS),
                ]/>
                <CodeBlock caption="Rust" html=snippets::EXAMPLE_RS/>
            </div>
        </section>
    }
}
