use blitz_traits::net::Url;

use super::LocalSource;

#[test]
fn a_local_source_reads_its_bytes() {
    assert_eq!(
        LocalSource::Data("data:,hi".into()).read().as_deref(),
        Some(&b"hi"[..])
    );
    assert_eq!(LocalSource::Data("data:no-comma".into()).read(), None);
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
