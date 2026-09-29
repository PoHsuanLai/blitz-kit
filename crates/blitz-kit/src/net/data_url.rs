//! Decoding a `data:` URI's payload.

/// Decode a `data:` URI's payload: `data:[<mediatype>][;base64],<data>` (RFC 2397). `None` for
/// anything that is not a well-formed `data:` URI.
pub(super) fn decode(uri: &str) -> Option<Vec<u8>> {
    let rest = uri.strip_prefix("data:")?;
    let comma = rest.find(',')?;
    let (meta, payload) = rest.split_at(comma);
    let payload = &payload[1..];
    if meta.ends_with(";base64") {
        base64_decode(payload)
    } else {
        Some(percent_decode(payload))
    }
}

/// A minimal RFC 4648 base64 decoder (standard alphabet, `=` padding). Not in the pinned
/// dependency block, so hand-rolled rather than adding a crate for a handful of small assets.
pub(super) fn base64_decode(input: &str) -> Option<Vec<u8>> {
    fn sextet(byte: u8) -> Option<u32> {
        match byte {
            b'A'..=b'Z' => Some(u32::from(byte - b'A')),
            b'a'..=b'z' => Some(u32::from(byte - b'a') + 26),
            b'0'..=b'9' => Some(u32::from(byte - b'0') + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let clean: Vec<u8> = input.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if !clean.len().is_multiple_of(4) {
        return None;
    }
    let mut out = Vec::with_capacity(clean.len() / 4 * 3);
    let (chunks, _remainder) = clean.as_chunks::<4>();
    for chunk in chunks {
        let pad = chunk.iter().rev().take_while(|&&b| b == b'=').count();
        let mut sextets = [0u32; 4];
        for (slot, &byte) in sextets.iter_mut().zip(chunk) {
            *slot = if byte == b'=' { 0 } else { sextet(byte)? };
        }
        let word = (sextets[0] << 18) | (sextets[1] << 12) | (sextets[2] << 6) | sextets[3];
        let bytes = [(word >> 16) as u8, (word >> 8) as u8, word as u8];
        out.extend_from_slice(&bytes[..3 - pad]);
    }
    Some(out)
}

/// Percent-decode a non-base64 payload (`%XX`); anything else passes through as raw bytes.
pub(super) fn percent_decode(input: &str) -> Vec<u8> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3])
                .ok()
                .and_then(|hex| u8::from_str_radix(hex, 16).ok());
            if let Some(value) = hex {
                out.push(value);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    out
}
