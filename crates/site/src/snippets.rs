//! Syntax-highlighted code snippets, generated at build time by `build.rs` from
//! the files in `snippets/`. Each constant holds highlighted inner HTML
//! (token spans with inline colors) rendered via `inner_html`.
include!(concat!(env!("OUT_DIR"), "/snippets.rs"));
