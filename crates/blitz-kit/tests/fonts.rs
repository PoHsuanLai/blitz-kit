//! A face registered on `SharedFonts` actually reaches a document (without it a document falls
//! back to fontconfig's faces): the family is reported, and a document naming that family in
//! `font-family` measures differently from one naming only `system-ui`.
//!
//! Uses a real font's bytes rather than a synthetic one: Inter, read from quire's bundled
//! assets if quire is checked out next to this repo, skipped (with a message, not a failure)
//! otherwise. The kit must not depend on quire, so this is a file read, never a dependency.

mod support;

use std::path::PathBuf;

use blitz_dom::FontContext;
use blitz_kit::fonts::{FontFaces, SharedFonts};

const INTER: &str = "inter-normal-400-700-latin.ttf";

/// Inter's bytes, leaked to `'static` (`FontFaces`' bound: a real shell's are `include_bytes!`
/// in its design-system crate), or `None` if quire is not checked out beside this repo.
fn inter_bytes() -> Option<&'static [u8]> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../quire/crates/ds/assets/fonts")
        .join(INTER);
    let bytes = std::fs::read(path).ok()?;
    Some(Box::leak(bytes.into_boxed_slice()))
}

/// The width `text` lays out at in `stack` under `fonts`.
fn width(fonts: FontContext, stack: &str, text: &str) -> f64 {
    let (mut doc, body) = support::page_with(400, 100, 1.0, Some(fonts));
    let style = format!("display:inline-block;white-space:pre;font-size:16px;font-family:{stack}");
    let line = support::div(&mut doc, body, "line", &style);
    support::text(&mut doc, line, text);
    doc.resolve(0.0);
    doc.get_client_bounding_rect(line)
        .expect("the line is laid out")
        .width
}

#[test]
fn a_registered_face_is_a_family() {
    let Some(bytes) = inter_bytes() else {
        eprintln!("skipped: quire is not checked out beside this repo");
        return;
    };
    let families = SharedFonts::system().register(bytes);
    assert!(
        families.contains(&"Inter".to_string()),
        "Inter is not in {families:?}"
    );
}

#[test]
fn no_faces_register_no_families() {
    let fonts = SharedFonts::system_with(&FontFaces::default());
    assert_eq!(format!("{fonts:?}"), "SharedFonts { .. }");
}

#[test]
fn a_registered_faces_stack_measures_differently_from_system_ui() {
    let Some(bytes) = inter_bytes() else {
        eprintln!("skipped: quire is not checked out beside this repo");
        return;
    };
    // The bug this pins: with only `SharedFonts::system()`, a font-family naming a bundled face
    // that is not also a system font fell straight through to `system-ui`, indistinguishable
    // from not naming it at all.
    let unregistered = width(
        SharedFonts::system().for_document(),
        "\"Inter\"",
        "shell-host",
    );
    let system = width(
        SharedFonts::system().for_document(),
        "system-ui,sans-serif",
        "shell-host",
    );
    if (unregistered - system).abs() >= 0.01 {
        eprintln!("skipped: Inter is a system font here, so it already resolved unregistered");
        return;
    }

    let registered = width(
        SharedFonts::system_with(&FontFaces(vec![bytes])).for_document(),
        "\"Inter\"",
        "shell-host",
    );
    assert!(
        (registered - system).abs() >= 1.0,
        "registering Inter did not change how \"Inter\" lays out ({registered} vs system-ui's {system})"
    );
}
