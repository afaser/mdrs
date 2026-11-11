use pulldown_cmark::{Event, Parser, Tag};

struct Rewrite {
    start: usize,
    end: usize,
    replacement: String,
}

pub fn resolve(source: &str, base: &std::path::Path) -> String {
    let rewrites = image_rewrites(source, base);

    if rewrites.is_empty() {
        return source.to_owned();
    }

    let mut resolved = String::with_capacity(source.len());
    let mut cursor = 0;

    for rewrite in rewrites {
        if rewrite.start < cursor {
            continue;
        }

        resolved.push_str(&source[cursor..rewrite.start]);
        resolved.push_str(&rewrite.replacement);
        cursor = rewrite.end;
    }

    resolved.push_str(&source[cursor..]);
    resolved
}

pub fn file_path(uri: &str) -> Option<std::path::PathBuf> {
    let encoded = uri.strip_prefix("file://")?;

    Some(std::path::PathBuf::from(decode_percent(encoded)))
}

fn image_rewrites(source: &str, base: &std::path::Path) -> Vec<Rewrite> {
    Parser::new(source)
        .into_offset_iter()
        .filter_map(|(event, range)| {
            let Event::Start(Tag::Image { .. }) = event else {
                return None;
            };

            let fragment = source.get(range.clone())?;
            let destination = destination_range(fragment)?;
            let start = range.start + destination.start;
            let end = range.start + destination.end;
            let raw = source.get(start..end)?;
            let replacement = resolve_uri(raw, base)?;

            Some(Rewrite {
                start,
                end,
                replacement,
            })
        })
        .collect()
}

fn destination_range(fragment: &str) -> Option<std::ops::Range<usize>> {
    let open = fragment.rfind("](")? + 2;
    let trimmed = fragment.get(open..)?.trim_start();
    let start = fragment.len() - trimmed.len();

    if trimmed.starts_with('<') {
        let end = trimmed.find('>')? + start;

        return Some(start + 1..end);
    }

    let mut depth = 0usize;
    let mut escaped = false;

    for (offset, character) in fragment.get(start..)?.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }

        match character {
            '\\' => escaped = true,
            '(' => depth += 1,
            ')' if depth == 0 => return Some(start..start + offset),
            ')' => depth -= 1,
            ' ' | '\t' | '\n' if depth == 0 => {
                return Some(start..start + offset);
            }
            _ => {}
        }
    }

    Some(start..fragment.len())
}

fn resolve_uri(uri: &str, base: &std::path::Path) -> Option<String> {
    if uri.is_empty() || uri.starts_with('#') || has_scheme(uri) {
        return None;
    }

    let decoded = decode_percent(uri);
    let path = std::path::Path::new(&decoded);
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    };
    let absolute = std::path::absolute(&joined).unwrap_or(joined);
    let encoded = encode_percent(&absolute.to_string_lossy());

    Some(format!("file://{encoded}"))
}

fn has_scheme(uri: &str) -> bool {
    uri.split_once(':').is_some_and(|(scheme, _)| {
        !scheme.is_empty()
            && scheme.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
            })
    })
}

fn decode_percent(uri: &str) -> String {
    let bytes = uri.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if let Some(value) = percent_byte(bytes, index) {
            decoded.push(value);
            index += 3;
            continue;
        }

        decoded.push(bytes[index]);
        index += 1;
    }

    String::from_utf8_lossy(&decoded).into_owned()
}

fn percent_byte(bytes: &[u8], index: usize) -> Option<u8> {
    if bytes.get(index) != Some(&b'%') {
        return None;
    }

    let high = hex(*bytes.get(index + 1)?)?;
    let low = hex(*bytes.get(index + 2)?)?;

    Some(high << 4 | low)
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
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'/') {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }

    encoded
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn resolved(source: &str) -> String {
        resolve(source, Path::new("/docs"))
    }

    #[test]
    fn relative_image_becomes_absolute_file_uri() {
        let source = "![plot](images/plot.png)";

        let output = resolved(source);

        assert_eq!(output, "![plot](file:///docs/images/plot.png)");
        assert_eq!(
            file_path("file:///docs/images/plot.png"),
            Some(std::path::PathBuf::from("/docs/images/plot.png"))
        );
    }

    #[test]
    fn external_image_is_unchanged() {
        let source = "![logo](https://example.com/logo.png)";

        assert_eq!(resolved(source), source);
    }

    #[test]
    fn data_image_is_unchanged() {
        let source = "![pixel](data:image/png;base64,AAAA)";

        assert_eq!(resolved(source), source);
    }

    #[test]
    fn anchor_image_is_unchanged() {
        let source = "![section](#section)";

        assert_eq!(resolved(source), source);
    }

    #[test]
    fn image_path_with_spaces_is_encoded() {
        let source = "![photo](<my image.png>)";

        let output = resolved(source);

        assert_eq!(output, "![photo](<file:///docs/my%20image.png>)");
        assert_eq!(
            file_path("file:///docs/my%20image.png"),
            Some(std::path::PathBuf::from("/docs/my image.png"))
        );
    }

    #[test]
    fn percent_encoded_image_path_is_decoded_before_joining() {
        let source = "![photo](my%20image.png)";

        let output = resolved(source);

        assert_eq!(output, "![photo](file:///docs/my%20image.png)");
    }

    #[test]
    fn absolute_image_path_becomes_file_uri() {
        let source = "![photo](/srv/photo.png)";

        assert_eq!(resolved(source), "![photo](file:///srv/photo.png)");
    }
}
