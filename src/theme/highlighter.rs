use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::BufReader;
use std::sync::{Arc, LazyLock};

use crate::text::Span;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Highlighter as SyntectHighlighter, Theme, ThemeSet};
use syntect::parsing::{ParseState, ScopeStack, SyntaxSet};
use two_face::theme::EmbeddedThemeName;

use crate::config::{Highlighting, Palette, TokenClass, Tokens};

static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(two_face::syntax::extra_no_newlines);
static THEMES: LazyLock<two_face::theme::EmbeddedLazyThemeSet> =
    LazyLock::new(two_face::theme::extra);

type Block = Arc<[Span<'static>]>;

pub(super) struct Highlighter {
    theme: Theme,
    tokens: Tokens,
    palette: Palette,
    cache: RefCell<HashMap<String, HashMap<u64, Block>>>,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new(&Highlighting::default(), &Palette::default())
    }
}

impl Highlighter {
    pub(super) fn new(highlighting: &Highlighting, palette: &Palette) -> Self {
        Self {
            theme: theme(highlighting),
            tokens: highlighting.tokens,
            palette: *palette,
            cache: RefCell::new(HashMap::new()),
        }
    }

    pub(super) fn block(&self, language: &str, code: &str) -> Block {
        let code_hash = hash(code);
        let cached = self
            .cache
            .borrow()
            .get(language)
            .and_then(|blocks| blocks.get(&code_hash))
            .cloned();

        if let Some(block) = cached {
            return block;
        }

        let block = self.highlight_block(language, code);
        self.cache
            .borrow_mut()
            .entry(language.to_owned())
            .or_default()
            .insert(code_hash, Arc::clone(&block));

        block
    }

    fn highlight_block(&self, language: &str, code: &str) -> Block {
        if self.tokens.is_empty() {
            return self.highlight_fast(language, code);
        }

        self.highlight_with_tokens(language, code)
    }

    fn highlight_fast(&self, language: &str, code: &str) -> Block {
        let mut highlighter = HighlightLines::new(syntax(language), &self.theme);
        let mut spans: Vec<Span<'static>> = Vec::new();

        for (index, line) in code.lines().enumerate() {
            if index > 0 {
                spans.push(Span::new("\n"));
            }

            match highlighter.highlight_line(line, &SYNTAXES) {
                Ok(ranges) => spans.extend(ranges.into_iter().map(|(style, text)| {
                    Span::new(text.to_owned()).color(color(style.foreground))
                })),
                Err(_) => spans.push(Span::new(line.to_owned())),
            }
        }

        Arc::from(spans.into_boxed_slice())
    }

    fn highlight_with_tokens(&self, language: &str, code: &str) -> Block {
        let mut parse_state = ParseState::new(syntax(language));
        let highlighter = SyntectHighlighter::new(&self.theme);
        let mut stack = ScopeStack::new();
        let mut spans: Vec<Span<'static>> = Vec::new();

        for (index, line) in code.lines().enumerate() {
            if index > 0 {
                spans.push(Span::new("\n"));
            }

            let Ok(changes) = parse_state.parse_line(line, &SYNTAXES) else {
                spans.push(Span::new(line.to_owned()));
                continue;
            };

            let mut position = 0;

            for (end, operation) in changes {
                let text = line.get(position..end).unwrap_or_default();
                self.push_span(&mut spans, text, &stack, &highlighter);
                let _ = stack.apply(&operation);
                position = end;
            }

            let text = line.get(position..).unwrap_or_default();
            self.push_span(&mut spans, text, &stack, &highlighter);
        }

        Arc::from(spans.into_boxed_slice())
    }

    fn push_span(
        &self,
        spans: &mut Vec<Span<'static>>,
        text: &str,
        stack: &ScopeStack,
        highlighter: &SyntectHighlighter<'_>,
    ) {
        if text.is_empty() {
            return;
        }

        let style = highlighter.style_for_stack(&stack.scopes);
        let foreground = self
            .token_color(stack)
            .unwrap_or_else(|| color(style.foreground));

        spans.push(Span::new(text.to_owned()).color(foreground));
    }

    fn token_color(&self, stack: &ScopeStack) -> Option<iced::Color> {
        let class = stack
            .scopes
            .iter()
            .rev()
            .find_map(|scope| class_for(&scope.build_string()))?;

        self.tokens
            .color(class)
            .map(|color| color.resolve(&self.palette))
    }
}

fn class_for(scope: &str) -> Option<TokenClass> {
    if scope.starts_with("keyword.operator") {
        return Some(TokenClass::Operator);
    }

    if scope.starts_with("keyword") {
        return Some(TokenClass::Keyword);
    }

    if scope.starts_with("string") {
        return Some(TokenClass::String);
    }

    if scope.starts_with("comment") {
        return Some(TokenClass::Comment);
    }

    if scope.starts_with("constant.numeric") {
        return Some(TokenClass::Number);
    }

    if scope.starts_with("entity.name.function") || scope.starts_with("support.function") {
        return Some(TokenClass::Function);
    }

    if scope.starts_with("entity.name.type")
        || scope.starts_with("storage.type")
        || scope.starts_with("support.type")
    {
        return Some(TokenClass::Type);
    }

    if scope.starts_with("punctuation") {
        return Some(TokenClass::Punctuation);
    }

    None
}

fn theme(highlighting: &Highlighting) -> Theme {
    if let Some(path) = &highlighting.theme_file {
        match std::fs::File::open(path) {
            Ok(file) => match ThemeSet::load_from_reader(&mut BufReader::new(file)) {
                Ok(theme) => return theme,
                Err(error) => log::warn!("highlighting: {}: {error}", path.display()),
            },
            Err(error) => log::warn!("highlighting: {}: {error}", path.display()),
        }
    }

    let name = two_face::theme::EmbeddedLazyThemeSet::theme_names()
        .iter()
        .copied()
        .find(|name| normalize(name.as_name()) == normalize(&highlighting.theme))
        .unwrap_or(EmbeddedThemeName::CatppuccinMocha);

    THEMES.get(name).clone()
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn hash(code: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    code.hash(&mut hasher);

    hasher.finish()
}

fn syntax(language: &str) -> &'static syntect::parsing::SyntaxReference {
    SYNTAXES
        .find_syntax_by_token(language)
        .unwrap_or_else(|| SYNTAXES.find_syntax_plain_text())
}

fn color(color: syntect::highlighting::Color) -> iced::Color {
    iced::Color::from_rgb8(color.r, color.g, color.b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::colors::ColorRef;

    fn block_text(block: &[Span<'static>]) -> String {
        block.iter().map(|span| span.text.as_ref()).collect()
    }

    #[test]
    fn known_language_produces_colored_tokens() {
        let highlighter = Highlighter::default();

        let block = highlighter.block("rust", "let value = 1;");

        assert!(block.iter().any(|span| span.color.is_some()));
        assert!(block.iter().any(|span| span.color != block[0].color));
    }

    #[test]
    fn unknown_language_falls_back_to_monospaced_text() {
        let highlighter = Highlighter::default();

        let block = highlighter.block("unknown-language", "plain text");

        assert_eq!(block_text(&block), "plain text");
        assert!(block.iter().all(|span| span.font.is_none()));
    }

    #[test]
    fn empty_code_produces_no_spans() {
        let highlighter = Highlighter::default();

        assert!(highlighter.block("rust", "").is_empty());
    }

    #[test]
    fn token_override_changes_keyword_color() {
        let highlighting = Highlighting {
            tokens: Tokens {
                keyword: ColorRef::parse("#ff0000"),
                ..Tokens::default()
            },
            ..Highlighting::default()
        };
        let highlighter = Highlighter::new(&highlighting, &Palette::default());

        let block = highlighter.block("rust", "if true {}");

        assert!(
            block
                .iter()
                .any(|span| span.color == Some(iced::Color::from_rgb8(0xff, 0x00, 0x00)))
        );
    }

    #[test]
    fn token_overrides_apply_to_string_and_comment() {
        let highlighting = Highlighting {
            tokens: Tokens {
                string: ColorRef::parse("#00ff00"),
                comment: ColorRef::parse("#0000ff"),
                ..Tokens::default()
            },
            ..Highlighting::default()
        };
        let highlighter = Highlighter::new(&highlighting, &Palette::default());

        let block = highlighter.block("rust", "// note\nlet x = \"hi\";");
        let colors: Vec<Option<iced::Color>> = block.iter().map(|span| span.color).collect();

        assert!(colors.contains(&Some(iced::Color::from_rgb8(0x00, 0xff, 0x00))));
        assert!(colors.contains(&Some(iced::Color::from_rgb8(0x00, 0x00, 0xff))));
    }

    #[test]
    fn theme_name_is_normalized() {
        assert_eq!(normalize("Catppuccin Mocha"), "catppuccinmocha");
        assert_eq!(normalize("catppuccin-mocha"), "catppuccinmocha");
    }
}
