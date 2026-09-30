//! Decoding a `data:` URL's body (RFC 2397): the payload after the first comma, percent-decoded,
//! then base64-decoded when the header ends in `;base64`. Both forms matter: a design system's
//! grain PNG is base64 and its icon masks are percent-encoded SVG.

/// The bytes `url` carries, or `None` when it is not a well-formed `data:` URL.
pub fn decode(url: &str) -> Option<Vec<u8>> {
    let rest = strip_scheme(url)?;
    let (header, body) = rest.split_once(',')?;
    let body = percent_decode(body.as_bytes())?;
    let is_base64 = header
        .rsplit(';')
        .next()
        .is_some_and(|last| last.trim().eq_ignore_ascii_case("base64"));
    if is_base64 {
        base64_decode(&body)
    } else {
        Some(body)
    }
}

/// `url` after a case-insensitive `data:`.
fn strip_scheme(url: &str) -> Option<&str> {
    let url = url.trim_start();
    let scheme = url.get(..5)?;
    scheme.eq_ignore_ascii_case("data:").then(|| &url[5..])
}

/// `%XX` escapes replaced by their byte; a malformed escape is an error.
fn percent_decode(input: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(input.len());
    let mut bytes = input.iter().copied();
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let high = hex_digit(bytes.next()?)?;
            let low = hex_digit(bytes.next()?)?;
            out.push((high << 4) | low);
        } else {
            out.push(byte);
        }
    }
    Some(out)
}

fn hex_digit(byte: u8) -> Option<u8> {
    char::from(byte)
        .to_digit(16)
        .and_then(|digit| u8::try_from(digit).ok())
}

/// Standard or URL-safe base64, padding optional, ASCII whitespace ignored.
fn base64_decode(input: &[u8]) -> Option<Vec<u8>> {
    let sextets = input
        .iter()
        .copied()
        .filter(|byte| !byte.is_ascii_whitespace())
        .take_while(|&byte| byte != b'=')
        .map(sextet)
        .collect::<Option<Vec<u8>>>()?;
    if sextets.len() % 4 == 1 {
        return None;
    }
    let bytes = sextets
        .chunks(4)
        .flat_map(|chunk| {
            let word = chunk.iter().enumerate().fold(0u32, |word, (n, &six)| {
                word | (u32::from(six) << (18 - 6 * n))
            });
            let [_, a, b, c] = word.to_be_bytes();
            [a, b, c].into_iter().take(chunk.len() - 1)
        })
        .collect();
    Some(bytes)
}

/// One base64 digit's value.
fn sextet(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' | b'-' => Some(62),
        b'/' | b'_' => Some(63),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::decode;

    const CASES: &[(&str, Option<&[u8]>)] = &[
        ("data:,hello", Some(b"hello")),
        ("data:text/plain,a%20b", Some(b"a b")),
        (
            "data:image/svg+xml,%3Csvg%3E%3C/svg%3E",
            Some(b"<svg></svg>"),
        ),
        ("data:text/plain;base64,aGVsbG8=", Some(b"hello")),
        ("DATA:;BASE64,aGVsbG8", Some(b"hello")),
        ("data:;base64,aGk=", Some(b"hi")),
        ("data:;base64,aA==", Some(b"h")),
        ("data:;base64,aGVs bG8=", Some(b"hello")),
        ("data:;base64,", Some(b"")),
        ("data:;base64,a", None),
        ("data:;base64,a*bc", None),
        ("data:,bad%2", None),
        ("data:,bad%zz", None),
        ("data:no-comma", None),
        ("file:///tmp/x", None),
        ("", None),
    ];

    #[test]
    fn decodes_every_case() {
        for (url, expected) in CASES {
            assert_eq!(decode(url).as_deref(), *expected, "{url:?} decoded wrongly");
        }
    }

    #[test]
    fn decodes_the_grain_png_header() {
        // The first bytes of any PNG, base64-encoded as the grain's data: URL is.
        let png = decode("data:image/png;base64,iVBORw0KGgo=");
        assert_eq!(png.as_deref(), Some(&b"\x89PNG\r\n\x1a\n"[..]));
    }
}
