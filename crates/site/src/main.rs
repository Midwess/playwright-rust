use leptos::mount::mount_to_body;

mod app;
mod components;
mod snippets;

fn main() {
    // Keep main thin: a future CSR->SSR switch only touches this entry + the
    // build config, never the components in app.rs / components/.
    mount_to_body(app::App);
}
