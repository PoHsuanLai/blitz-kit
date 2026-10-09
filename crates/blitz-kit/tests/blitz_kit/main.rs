//! All integration tests, one binary. No test here sets process-global state, so none needs its
//! own process.

#[cfg(feature = "adapter")]
mod adapter;
mod fonts;
mod hit;
mod hover_sync;
mod net;
mod paint_rect;
mod snap;
mod support;

/// Fails when a test file is added but not included above, or left as its own binary beside
/// this one: every `tests/blitz_kit/*.rs` must be a `mod` here, and `tests/*.rs` must not exist.
#[test]
fn every_test_file_is_included() {
    use std::fs;
    use std::path::PathBuf;

    let tests = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
    let rs_files = |dir: PathBuf| -> Vec<String> {
        fs::read_dir(dir)
            .expect("tests dir is readable")
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "rs"))
            .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
            .collect()
    };
    assert_eq!(
        rs_files(tests.clone()),
        Vec::<String>::new(),
        "a tests/*.rs file would be its own binary; move it into tests/blitz_kit"
    );
    let main = fs::read_to_string(tests.join("blitz_kit/main.rs")).expect("main.rs");
    for name in rs_files(tests.join("blitz_kit")) {
        assert!(
            name == "main" || main.contains(&format!("mod {name};")),
            "tests/blitz_kit/{name}.rs is not included in main.rs"
        );
    }
}
