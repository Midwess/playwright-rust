//! Highlights the code snippets in `snippets/` at build time with syntect and
//! emits an `OUT_DIR/snippets.rs` module of `&str` constants holding the
//! highlighted inner HTML (token `<span>`s with inline colors, no outer
//! `<pre>`). The browser ships none of syntect; `src/snippets.rs` includes the
//! generated file and the components render the constants via `inner_html`.

use std::{env, fs, path::Path};

use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

fn main() {
    println!("cargo:rerun-if-changed=snippets");

    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];

    let mut paths: Vec<_> = fs::read_dir("snippets")
        .expect("snippets/ dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file())
        .collect();
    paths.sort();

    let mut generated = String::new();
    for path in paths {
        println!("cargo:rerun-if-changed={}", path.display());
        let code = fs::read_to_string(&path).expect("read snippet");
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let syntax = ss
            .find_syntax_by_extension(ext)
            .unwrap_or_else(|| ss.find_syntax_plain_text());
        let html = highlighted_html_for_string(&code, &ss, syntax, theme).expect("highlight");

        // Drop syntect's outer `<pre style=...>` / `</pre>`; the component
        // supplies its own styled <pre>. The first '>' closes the <pre> tag.
        // Trim the newline syntect emits right after `<pre>` (and before
        // `</pre>`) so the block has no leading/trailing blank line.
        let after_open = html.find('>').map(|i| &html[i + 1..]).unwrap_or(&html);
        let inner = after_open
            .trim()
            .strip_suffix("</pre>")
            .unwrap_or(after_open)
            .trim()
            .to_string();

        let stem = path.file_stem().unwrap().to_str().unwrap();
        let name = format!("{stem}_{ext}")
            .to_uppercase()
            .replace(['-', '.'], "_");
        generated.push_str(&format!("pub const {name}: &str = r####\"{inner}\"####;\n"));
    }

    let dest = Path::new(&env::var("OUT_DIR").unwrap()).join("snippets.rs");
    fs::write(dest, generated).expect("write snippets.rs");
}
