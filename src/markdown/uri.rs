use std::fmt::Write;
use std::path::{Path, PathBuf};

pub fn file_path(uri: &str) -> Option<PathBuf> {
    let encoded = uri.strip_prefix("file://")?;

    Some(PathBuf::from(decode_percent(encoded)))
}

pub(super) fn resolve_uri(uri: &str, base: &Path) -> Option<String> {
    if uri.is_empty() || uri.starts_with('#') || has_scheme(uri) {
        return None;
    }

    let decoded = decode_percent(uri);
    let path = Path::new(&decoded);
    let joined = base.join(path);
    let absolute = std::path::absolute(&joined).unwrap_or(joined);
    let encoded = encode_percent(&absolute.to_string_lossy());

    Some(format!("file://{encoded}"))
}

pub(super) fn has_scheme(uri: &str) -> bool {
    uri.split_once(':').is_some_and(|(scheme, _)| {
        !scheme.is_empty()
            && scheme.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
            })
    })
}

fn decode_percent(uri: &str) -> String {
    let bytes = uri.as_bytes();
    let mut decoded: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut rest = bytes;

    while let Some((&first, tail)) = rest.split_first() {
        let Some(value) = percent_byte(tail) else {
            decoded.push(first);
            rest = tail;
            continue;
        };

        decoded.push(value);
        rest = tail.get(2..).unwrap_or_default();
    }

    String::from_utf8_lossy(&decoded).into_owned()
}

fn percent_byte(rest: &[u8]) -> Option<u8> {
    let (&high, rest) = rest.split_first()?;
    let (&low, _) = rest.split_first()?;

    Some(hex(high)? << 4 | hex(low)?)
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn encode_percent(path: &str) -> String {
    let mut encoded = String::with_capacity(path.len());

    for byte in path.bytes() {
        if is_unreserved(byte) {
            encoded.push(char::from(byte));
            continue;
        }

        let _ = write!(encoded, "%{byte:02X}");
    }

    encoded
}

fn is_unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'/')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_path_rejects_non_file_uri() {
        assert_eq!(file_path("https://example.com/image.png"), None);
        assert_eq!(file_path("#section"), None);
        assert_eq!(file_path(""), None);
    }
}
