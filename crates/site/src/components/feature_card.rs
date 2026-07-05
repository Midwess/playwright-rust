use leptos::prelude::*;

/// One feature highlight: title, one-line blurb, and a code area supplied as
/// children (a `CodeBlock` for a single snippet, or `CodeTabs` for variants).
///
/// Set `unreleased` for a feature that's on the dev (main HEAD) build but not
/// yet on crates.io: the card shows an "Unreleased" badge and renders **only**
/// on the dev build (`SITE_VERSION == "dev"`), so release snapshots omit it.
/// When the feature ships, drop `unreleased` and the card becomes permanent.
#[component]
pub fn FeatureCard(
    /// Stable id so the dogfood test can assert the card rendered.
    id: &'static str,
    title: &'static str,
    blurb: &'static str,
    /// Mark a not-yet-released feature (dev-only, badged).
    #[prop(optional)]
    unreleased: bool,
    children: Children,
) -> impl IntoView {
    // An unreleased card only appears on the dev build.
    if unreleased && env!("SITE_VERSION") != "dev" {
        return ().into_any();
    }

    view! {
        <div id=id class="flex flex-col rounded-xl border border-rust-700/30 bg-ink-800 p-5">
            <div class="flex items-center gap-2">
                <h3 class="text-lg font-semibold text-rust-300">{title}</h3>
                {unreleased
                    .then(|| {
                        view! {
                            <span class="rounded-full border border-rust-500/50 bg-rust-500/10 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wider text-rust-300">
                                "Unreleased"
                            </span>
                        }
                    })}
            </div>
            <p class="mt-1 mb-4 text-sm text-rust-50/70">{blurb}</p>
            <div class="mt-auto">{children()}</div>
        </div>
    }
    .into_any()
}
