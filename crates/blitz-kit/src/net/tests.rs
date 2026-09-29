use blitz_traits::net::Url;

use super::LocalSource;
use super::data_url::{base64_decode, decode as decode_data_url, percent_decode};

#[test]
fn base64_round_trips_known_vectors() {
    let cases: &[(&str, &[u8])] = &[
        ("", b""),
        ("Zg==", b"f"),
        ("Zm8=", b"fo"),
        ("Zm9v", b"foo"),
        ("aGVsbG8=", b"hello"),
    ];
    for (encoded, want) in cases {
        assert_eq!(base64_decode(encoded).unwrap(), *want, "{encoded}");
    }
}

#[test]
fn base64_rejects_malformed_input() {
    assert_eq!(base64_decode("a"), None);
    assert_eq!(base64_decode("a!=="), None);
}

#[test]
fn percent_decode_unescapes_and_passes_through() {
    assert_eq!(percent_decode("Hello%2C%20world%21"), b"Hello, world!");
    assert_eq!(percent_decode("plain text"), b"plain text");
}

#[test]
fn data_url_splits_metadata_from_payload() {
    assert_eq!(
        decode_data_url("data:text/plain;base64,aGVsbG8=").unwrap(),
        b"hello"
    );
    assert_eq!(
        decode_data_url("data:text/plain,hello%20world").unwrap(),
        b"hello world"
    );
    assert_eq!(decode_data_url("http://example.com"), None);
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
