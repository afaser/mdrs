use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use iced::Pixels;
use iced::font::Style;
use iced::widget::markdown::Uri;
use iced::widget::text::Span;

use crate::config::{Role, Script};
use crate::theme::font::FontBook;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoleKey {
    Heading(usize),
    Paragraph,
    Code,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextRun<'a> {
    pub text: &'a str,
    pub script: Script,
}

pub(crate) type SpanCache = HashMap<(usize, RoleKey), Arc<[Span<'static, Uri>]>>;

pub fn runs(text: &str) -> Vec<TextRun<'_>> {
    let mut runs = Vec::new();
    let mut start = 0;
    let mut current: Option<Script> = None;

    for (index, character) in text.char_indices() {
        let script = Script::of(character);

        if script == Script::Common {
            continue;
        }

        match current {
            None => current = Some(script),
            Some(active) if active != script => {
                runs.push(TextRun {
                    text: &text[start..index],
                    script: active,
                });
                start = index;
                current = Some(script);
            }
            Some(_) => {}
        }
    }

    if start < text.len() {
        runs.push(TextRun {
            text: &text[start..],
            script: current.unwrap_or(Script::Common),
        });
    }

    runs
}

#[derive(Default)]
pub struct SpanStyler {
    cache: RefCell<SpanCache>,
}

impl SpanStyler {
    pub fn style(
        &self,
        spans: &Arc<[Span<'static, Uri>]>,
        role: &Role,
        key: RoleKey,
        book: &FontBook,
    ) -> Arc<[Span<'static, Uri>]> {
        let cache_key = (Arc::as_ptr(spans) as *const () as usize, key);

        if let Some(cached) = self.cache.borrow().get(&cache_key) {
            return Arc::clone(cached);
        }

        let styled = style_spans(spans, role, book);
        self.cache
            .borrow_mut()
            .insert(cache_key, Arc::clone(&styled));
        styled
    }

    pub fn clear(&self) {
        self.cache.borrow_mut().clear();
    }
}

fn style_spans(
    spans: &[Span<'static, Uri>],
    role: &Role,
    book: &FontBook,
) -> Arc<[Span<'static, Uri>]> {
    let base = book.role_font(role, role.weight, Style::Normal);

    spans
        .iter()
        .flat_map(|span| style_span(span, role, book, base.as_ref()))
        .collect::<Vec<_>>()
        .into()
}

fn style_span(
    span: &Span<'static, Uri>,
    role: &Role,
    book: &FontBook,
    base: Option<&iced::Font>,
) -> Vec<Span<'static, Uri>> {
    let explicit = span.font;
    let weight = explicit.map(|font| font.weight).unwrap_or(role.weight);
    let style = explicit.map(|font| font.style).unwrap_or(Style::Normal);

    if let Some(font) = explicit
        && base.is_some_and(|base| base.family != font.family)
    {
        let mut span = span.clone();
        span.letter_spacing = (role.letter_spacing > 0.0).then_some(Pixels(role.letter_spacing));
        return vec![span];
    }

    let runs = runs(span.text.as_ref());
    let mut styled = Vec::new();

    for run in runs {
        let Some(font) = book.role_script_font(role, run.script, weight, style) else {
            return vec![span.clone()];
        };
        let mut span = span.clone();
        span.text = std::borrow::Cow::Owned(run.text.to_owned());
        span.font = Some(font);
        span.letter_spacing = (role.letter_spacing > 0.0).then_some(Pixels(role.letter_spacing));
        styled.push(span);
    }

    styled
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Fonts, Role};
    use crate::theme::font::FontBook;
    use iced::font::Weight;

    #[test]
    fn mixed_text_splits_into_script_runs() {
        let runs = runs("Hello, мир");
        let texts: Vec<&str> = runs.iter().map(|run| run.text).collect();

        assert_eq!(texts, vec!["Hello, ", "мир"]);
        assert_eq!(runs[0].script, Script::Latin);
        assert_eq!(runs[1].script, Script::Cyrillic);
    }

    #[test]
    fn all_latin_stays_single_run() {
        let runs = runs("Hello");

        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "Hello");
        assert_eq!(runs[0].script, Script::Latin);
    }

    #[test]
    fn all_cyrillic_stays_single_run() {
        let runs = runs("Привет");

        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].script, Script::Cyrillic);
    }

    #[test]
    fn bold_and_italic_keep_weight_and_style() {
        let book = FontBook::new(&Fonts::default());
        let role = Role {
            font_set: None,
            fonts: crate::config::FontChain::new(vec!["latin".to_owned(), "cyrillic".to_owned()]),
            size: 17.0,
            weight: Weight::Normal,
            line_height: 1.3,
            letter_spacing: 0.0,
            color: crate::config::colors::ColorRef::Palette(
                crate::config::palette::PaletteKey::Text,
            ),
            ligatures: false,
        };
        let base = book
            .chain_font(&role.fonts, Weight::Normal, Style::Normal)
            .expect("font");
        let bold = Span::new("жирный".to_owned()).font(iced::Font {
            weight: Weight::Bold,
            ..base
        });
        let italic = Span::new("italic".to_owned()).font(iced::Font {
            style: Style::Italic,
            ..base
        });
        let spans: Arc<[Span<'static, Uri>]> = Arc::from(vec![bold, italic].into_boxed_slice());

        let styled = style_spans(&spans, &role, &book);

        assert_eq!(styled[0].font.map(|font| font.weight), Some(Weight::Bold));
        assert_eq!(styled[1].font.map(|font| font.style), Some(Style::Italic));
    }

    #[test]
    fn inline_code_span_is_not_resplit() {
        let book = FontBook::new(&Fonts::default());
        let role = Role {
            font_set: None,
            fonts: crate::config::FontChain::new(vec!["latin".to_owned(), "cyrillic".to_owned()]),
            size: 17.0,
            weight: Weight::Normal,
            line_height: 1.3,
            letter_spacing: 0.0,
            color: crate::config::colors::ColorRef::Palette(
                crate::config::palette::PaletteKey::Text,
            ),
            ligatures: false,
        };
        let code_font = book
            .chain_font(
                &crate::config::FontChain::new(vec!["code".to_owned()]),
                Weight::Normal,
                Style::Normal,
            )
            .expect("code font");
        let span = Span::new("Hello, мир".to_owned()).font(code_font);
        let spans: Arc<[Span<'static, Uri>]> = Arc::from(vec![span].into_boxed_slice());

        let styled = style_spans(&spans, &role, &book);

        assert_eq!(styled.len(), 1);
        assert_eq!(styled[0].font, Some(code_font));
        assert!(styled[0].letter_spacing.is_none());
    }

    #[test]
    fn role_letter_spacing_is_written_to_spans() {
        let book = FontBook::new(&Fonts::default());
        let mut role = Role {
            font_set: None,
            fonts: crate::config::FontChain::new(vec!["latin".to_owned()]),
            size: 17.0,
            weight: Weight::Normal,
            line_height: 1.3,
            letter_spacing: 0.5,
            color: crate::config::colors::ColorRef::Palette(
                crate::config::palette::PaletteKey::Text,
            ),
            ligatures: false,
        };
        let span = Span::new("text".to_owned());
        let spans: Arc<[Span<'static, Uri>]> = Arc::from(vec![span].into_boxed_slice());

        let styled = style_spans(&spans, &role, &book);
        role.letter_spacing = 0.0;
        let plain = style_spans(&spans, &role, &book);

        assert_eq!(styled[0].letter_spacing, Some(iced::Pixels(0.5)));
        assert!(plain[0].letter_spacing.is_none());
    }

    #[test]
    fn spans_get_script_fonts_and_preserve_links() {
        let book = FontBook::new(&Fonts::default());
        let role = Role {
            font_set: None,
            fonts: crate::config::FontChain::new(vec!["latin".to_owned(), "cyrillic".to_owned()]),
            size: 17.0,
            weight: Weight::Normal,
            line_height: 1.3,
            letter_spacing: 0.0,
            color: crate::config::colors::ColorRef::Palette(
                crate::config::palette::PaletteKey::Text,
            ),
            ligatures: false,
        };
        let span = Span::new("Hello, мир".to_owned()).link("https://example.com".to_owned());
        let spans: Arc<[Span<'static, Uri>]> = Arc::from(vec![span].into_boxed_slice());

        let styled = style_spans(&spans, &role, &book);

        assert_eq!(styled.len(), 2);
        assert_eq!(styled[0].text.as_ref(), "Hello, ");
        assert_eq!(styled[1].text.as_ref(), "мир");
        assert_eq!(styled[0].link.as_deref(), styled[1].link.as_deref());
        assert_ne!(styled[0].font, styled[1].font);
    }
}
