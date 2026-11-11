use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::{Arc, LazyLock};

use iced::widget::markdown::Uri;
use iced::widget::text::Span;
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use two_face::theme::EmbeddedThemeName;

static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(two_face::syntax::extra_no_newlines);
static THEMES: LazyLock<two_face::theme::EmbeddedLazyThemeSet> =
    LazyLock::new(two_face::theme::extra);

type Line = Arc<[Span<'static, Uri>]>;

#[derive(Hash, PartialEq, Eq)]
struct Key {
    language: String,
    code_hash: u64,
}

#[derive(Default)]
pub struct Highlighter {
    cache: RefCell<HashMap<Key, Vec<Line>>>,
    blocks: RefCell<HashMap<Key, Arc<[Span<'static, Uri>]>>>,
}

impl Highlighter {
    pub fn block(&self, language: &str, code: &str) -> Arc<[Span<'static, Uri>]> {
        let key = Key {
            language: language.to_owned(),
            code_hash: hash(code),
        };
        let cached = self.blocks.borrow().get(&key).cloned();

        if let Some(block) = cached {
            return block;
        }

        let lines = self.lines(language, code);
        let mut spans = Vec::new();

        for (index, line) in lines.iter().enumerate() {
            if index > 0 {
                spans.push(Span::new("\n"));
            }

            spans.extend(line.iter().cloned());
        }

        let block: Arc<[Span<'static, Uri>]> = Arc::from(spans.into_boxed_slice());
        self.blocks.borrow_mut().insert(key, block.clone());

        block
    }

    pub fn lines(&self, language: &str, code: &str) -> Vec<Line> {
        let key = Key {
            language: language.to_owned(),
            code_hash: hash(code),
        };
        let cached = self.cache.borrow().get(&key).cloned();

        if let Some(lines) = cached {
            return lines;
        }

        let lines = highlight(language, code);
        self.cache.borrow_mut().insert(key, lines.clone());

        lines
    }
}

fn hash(code: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    code.hash(&mut hasher);

    hasher.finish()
}

fn highlight(language: &str, code: &str) -> Vec<Line> {
    let syntax = SYNTAXES
        .find_syntax_by_token(language)
        .unwrap_or_else(|| SYNTAXES.find_syntax_plain_text());
    let theme = THEMES.get(EmbeddedThemeName::CatppuccinMocha);
    let mut highlighter = HighlightLines::new(syntax, theme);

    code.lines()
        .map(|line| match highlighter.highlight_line(line, &SYNTAXES) {
            Ok(ranges) => ranges
                .into_iter()
                .map(|(style, text)| Span::new(text.to_owned()).color(color(style.foreground)))
                .collect::<Vec<_>>(),
            Err(_) => vec![Span::new(line.to_owned())],
        })
        .map(|spans| Arc::from(spans.into_boxed_slice()))
        .collect()
}

fn color(color: syntect::highlighting::Color) -> iced::Color {
    iced::Color::from_rgb8(color.r, color.g, color.b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_language_produces_colored_tokens() {
        let highlighter = Highlighter::default();

        let lines = highlighter.lines("rust", "let value = 1;");

        assert_eq!(lines.len(), 1);
        assert!(lines[0].iter().any(|span| span.color.is_some()));
        assert!(lines[0].iter().any(|span| span.color != lines[0][0].color));
    }

    #[test]
    fn unknown_language_falls_back_to_monospaced_text() {
        let highlighter = Highlighter::default();

        let lines = highlighter.lines("unknown-language", "plain text");

        let text: String = lines[0].iter().map(|span| span.text.as_ref()).collect();

        assert_eq!(text, "plain text");
        assert!(lines[0].iter().all(|span| span.font.is_none()));
    }

    #[test]
    fn repeated_lookup_returns_identical_cache() {
        let highlighter = Highlighter::default();

        let first = highlighter.lines("rust", "let value = 1;");
        let second = highlighter.lines("rust", "let value = 1;");

        assert!(Arc::ptr_eq(&first[0], &second[0]));
    }

    #[test]
    fn empty_code_produces_no_lines() {
        let highlighter = Highlighter::default();

        assert!(highlighter.lines("rust", "").is_empty());
    }
}
