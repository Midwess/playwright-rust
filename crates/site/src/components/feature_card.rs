use leptos::prelude::*;

/// One feature highlight: title, one-line blurb, and a code area supplied as
/// children (a `CodeBlock` for a single snippet, or `CodeTabs` for variants).
#[component]
pub fn FeatureCard(
    /// Stable id so the dogfood test can assert the card rendered.
    id: &'static str,
    title: &'static str,
    blurb: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            id=id
            class="flex flex-col rounded-xl border border-rust-700/30 bg-ink-800 p-5"
        >
            <h3 class="text-lg font-semibold text-rust-300">{title}</h3>
            <p class="mt-1 mb-4 text-sm text-rust-50/70">{blurb}</p>
            <div class="mt-auto">{children()}</div>
        </div>
    }
}
