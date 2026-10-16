use blitz_traits::net::Url;

use super::LocalSource;

#[test]
fn a_local_source_that_cannot_be_read_gives_nothing() {
    assert_eq!(
        LocalSource::File("/nonexistent/blitz-kit".into()).read(),
        None
    );
    assert_eq!(LocalSource::Unservable.read(), None);
}

#[test]
fn requests_classify_by_scheme_and_path() {
    let cases: &[(&str, LocalSource)] = &[
        (
            "data:text/plain,hi",
            LocalSource::Data("data:text/plain,hi".into()),
        ),
        (
            "file:///usr/share/backgrounds/a.png",
            LocalSource::File("/usr/share/backgrounds/a.png".into()),
        ),
        (
            "file://localhost/tmp/a%20b.png",
            LocalSource::File("/tmp/a b.png".into()),
        ),
        ("file://elsewhere/tmp/a.png", LocalSource::Unservable),
        ("https://example.com/a.png", LocalSource::Unservable),
    ];
    for (url, want) in cases {
        let parsed: Url = url.parse().expect(url);
        assert_eq!(LocalSource::of(&parsed), *want, "{url}");
    }
}

mod scratch {
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU32, Ordering};

    /// A private directory under the temp dir, removed on drop.
    pub(super) struct Dir(PathBuf);

    impl Dir {
        pub(super) fn new() -> Dir {
            static NEXT: AtomicU32 = AtomicU32::new(0);
            let n = NEXT.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("blitz-kit-net-{}-{n}", std::process::id()));
            std::fs::create_dir_all(&path).expect("scratch dir");
            Dir(path)
        }

        pub(super) fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

use super::source::read_regular;
use super::{FileRoots, MAX_RESOURCE_BYTES};

#[test]
fn only_a_regular_file_within_the_cap_is_read() {
    let dir = scratch::Dir::new();
    let file = dir.path().join("ten.bin");
    std::fs::write(&file, [7u8; 10]).expect("file");
    let link = dir.path().join("link.bin");
    std::os::unix::fs::symlink(&file, &link).expect("symlink");

    let cases: &[(&str, &std::path::Path, u64, Option<usize>)] = &[
        ("at the cap", &file, 10, Some(10)),
        ("one byte over the cap", &file, 9, None),
        ("a directory", dir.path(), 10, None),
        ("a symlink, even to a small file", &link, 10, None),
    ];
    for (name, path, cap, want) in cases {
        assert_eq!(
            read_regular(path, *cap).map(|bytes| bytes.len()),
            *want,
            "{name}"
        );
    }
}

#[test]
fn a_pipe_is_refused_without_opening_it() {
    let dir = scratch::Dir::new();
    let fifo = dir.path().join("pipe");
    let made = std::process::Command::new("mkfifo").arg(&fifo).status();
    if !made.is_ok_and(|status| status.success()) {
        return;
    }
    // Opening a pipe nobody writes to would block this thread forever.
    assert_eq!(read_regular(&fifo, MAX_RESOURCE_BYTES), None);
}

#[test]
fn an_oversized_data_url_is_refused() {
    let body = "a".repeat(usize::try_from(MAX_RESOURCE_BYTES).expect("fits") + 1);
    let huge = LocalSource::Data(format!("data:text/plain,{body}"));
    assert_eq!(huge.read(), None);
    let small = LocalSource::Data("data:text/plain,hi".into());
    assert_eq!(small.read(), Some(b"hi".to_vec()));
}

#[test]
fn a_file_is_read_only_inside_the_roots() {
    let dir = scratch::Dir::new();
    let inside = dir.path().join("pics");
    let outside = dir.path().join("pics-evil");
    std::fs::create_dir_all(&inside).expect("root");
    std::fs::create_dir_all(&outside).expect("lookalike");
    std::fs::write(inside.join("a.png"), b"inside").expect("file");
    std::fs::write(outside.join("b.png"), b"outside").expect("file");
    std::os::unix::fs::symlink(outside.join("b.png"), inside.join("sneaky.png")).expect("link");
    let roots = FileRoots::new([inside.clone()]);

    let cases: &[(&str, std::path::PathBuf, Option<&[u8]>)] = &[
        ("in the root", inside.join("a.png"), Some(b"inside")),
        ("beside the root", outside.join("b.png"), None),
        (
            "back out through ..",
            inside.join("..").join("pics-evil").join("b.png"),
            None,
        ),
        ("a symlink out of the root", inside.join("sneaky.png"), None),
        ("a missing file", inside.join("none.png"), None),
    ];
    for (name, path, want) in cases {
        let got = LocalSource::File(path.clone()).read_within(&roots);
        assert_eq!(got.as_deref(), *want, "{name}");
    }
    let none = FileRoots::default();
    assert_eq!(
        LocalSource::File(inside.join("a.png")).read_within(&none),
        None,
        "a consumer that names no roots serves no files"
    );
}
