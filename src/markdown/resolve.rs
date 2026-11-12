use std::borrow::Cow;
use std::ops::Range;
use std::path::Path;

use pulldown_cmark::{Event, Parser, Tag};

use crate::markdown::uri::resolve_uri;

struct Rewrite {
    from: usize,
    to: usize,
    replacement: String,
}

pub fn resolve<'a>(source: &'a str, base: &Path) -> Cow<'a, str> {
    let rewrites = image_rewrites(source, base);

    if rewrites.is_empty() {
        return Cow::Borrowed(source);
    }

    Cow::Owned(apply_rewrites(source, rewrites))
}

fn image_rewrites(source: &str, base: &Path) -> Vec<Rewrite> {
    Parser::new(source)
        .into_offset_iter()
        .filter_map(|(event, range)| rewrite(source, base, event, range))
        .collect()
}

fn rewrite(source: &str, base: &Path, event: Event<'_>, range: Range<usize>) -> Option<Rewrite> {
    let Event::Start(Tag::Image { .. }) = event else {
        return None;
    };

    let fragment = source.get(range.clone())?;
    let destination = destination_range(fragment)?;
    let from = range.start + destination.start;
    let to = range.start + destination.end;
    let raw = source.get(from..to)?;
    let replacement = resolve_uri(raw, base)?;

    Some(Rewrite {
        from,
        to,
        replacement,
    })
}

fn apply_rewrites(source: &str, rewrites: Vec<Rewrite>) -> String {
    let mut resolved = String::with_capacity(source.len());
    let mut cursor = 0;

    for rewrite in rewrites {
        if rewrite.from < cursor {
            continue;
        }

        resolved.push_str(&source[cursor..rewrite.from]);
        resolved.push_str(&rewrite.replacement);
        cursor = rewrite.to;
    }

    resolved.push_str(&source[cursor..]);
    resolved
}

fn destination_range(fragment: &str) -> Option<Range<usize>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn resolved(source: &str) -> String {
        resolve(source, Path::new("/docs")).into_owned()
    }

    #[test]
    fn relative_image_becomes_absolute_file_uri() {
        let source = "![plot](images/plot.png)";

        let output = resolved(source);

        assert_eq!(output, "![plot](file:///docs/images/plot.png)");
        assert_eq!(
            crate::markdown::file_path("file:///docs/images/plot.png"),
            Some(std::path::PathBuf::from("/docs/images/plot.png"))
        );
        assert_eq!(
            resolved("![logo](https://example.com/logo.png)"),
            "![logo](https://example.com/logo.png)"
        );
        assert_eq!(
            resolved("![pixel](data:image/png;base64,AAAA)"),
            "![pixel](data:image/png;base64,AAAA)"
        );
        assert_eq!(resolved("![section](#section)"), "![section](#section)");
    }

    #[test]
    fn image_path_with_spaces_is_encoded() {
        let source = "![photo](<my image.png>)";

        let output = resolved(source);

        assert_eq!(output, "![photo](<file:///docs/my%20image.png>)");
        assert_eq!(
            crate::markdown::file_path("file:///docs/my%20image.png"),
            Some(std::path::PathBuf::from("/docs/my image.png"))
        );
    }

    #[test]
    fn percent_encoded_image_path_is_decoded_before_joining() {
        let source = "![photo](my%20image.png)";

        assert_eq!(resolved(source), "![photo](file:///docs/my%20image.png)");
    }

    #[test]
    fn absolute_image_path_becomes_file_uri() {
        let source = "![photo](/srv/photo.png)";

        assert_eq!(resolved(source), "![photo](file:///srv/photo.png)");
    }

    #[test]
    fn empty_source_is_unchanged() {
        assert_eq!(resolved(""), "");
    }

    #[test]
    fn multiple_images_are_rewritten() {
        let source = "![one](one.png)\n\n![two](two.png)";

        assert_eq!(
            resolved(source),
            "![one](file:///docs/one.png)\n\n![two](file:///docs/two.png)"
        );
    }
}
