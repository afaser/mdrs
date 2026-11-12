use iced::Element;
use iced::widget::column;

use crate::markdown::Cull;
use crate::markdown::item::Item;
use crate::markdown::settings::Settings;
use crate::markdown::viewer::Viewer;

pub fn view_with<'a, Message: 'a>(
    items: impl IntoIterator<Item = &'a Item>,
    settings: Settings,
    viewer: &impl Viewer<'a, Message>,
) -> Element<'a, Message> {
    let blocks = items
        .into_iter()
        .enumerate()
        .map(|(index, item)| self::item(viewer, settings, item, index));

    column(blocks).spacing(settings.spacing).into()
}

pub fn item<'a, Message: 'a>(
    viewer: &impl Viewer<'a, Message>,
    settings: Settings,
    item: &'a Item,
    index: usize,
) -> Element<'a, Message> {
    let content = match item {
        Item::Image { url, title, alt } => viewer.image(settings, url, title, alt),
        Item::Heading(level, text) => viewer.heading(settings, level, text, index),
        Item::Paragraph(text) => viewer.paragraph(settings, text),
        Item::CodeBlock {
            language,
            code,
            lines,
        } => viewer.code_block(settings, language.as_deref(), code, lines),
        Item::List {
            start: None,
            bullets,
        } => viewer.unordered_list(settings, bullets),
        Item::List {
            start: Some(start),
            bullets,
        } => viewer.ordered_list(settings, *start, bullets),
        Item::Quote(contents) => viewer.quote(settings, contents),
        Item::Rule => viewer.rule(settings),
        Item::Table { columns, rows } => viewer.table(settings, columns, rows),
    };

    Cull::new(content).into()
}

pub(crate) fn items<'a, Message: 'a>(
    viewer: &impl Viewer<'a, Message>,
    settings: Settings,
    contents: &'a [Item],
) -> Element<'a, Message> {
    view_with(contents, settings, viewer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::{Background, Border, Color, Font, Padding, Pixels};

    use crate::markdown::Style;
    use crate::text::Highlight;

    struct DefaultViewer;

    impl<'a> Viewer<'a, ()> for DefaultViewer {}

    fn settings() -> Settings {
        Settings {
            text_size: Pixels(16.0),
            h1_size: Pixels(32.0),
            h2_size: Pixels(28.0),
            h3_size: Pixels(24.0),
            h4_size: Pixels(20.0),
            h5_size: Pixels(16.0),
            h6_size: Pixels(16.0),
            code_size: Pixels(12.0),
            spacing: Pixels(8.0),
            style: Style {
                font: Font::default(),
                inline_code_highlight: Highlight {
                    background: Background::Color(Color::BLACK),
                    border: Border::default(),
                },
                inline_code_padding: Padding::default(),
                inline_code_color: Color::WHITE,
                inline_code_font: Font::MONOSPACE,
                code_block_font: Font::MONOSPACE,
                link_color: Color::from_rgb8(0, 0, 255),
            },
        }
    }

    fn builds(source: &str) {
        let items = crate::markdown::parse(source);
        let _element = view_with(&items, settings(), &DefaultViewer);
    }

    #[test]
    fn view_with_accepts_every_item_variant() {
        builds(
            "# Heading\n\nParagraph with [link](https://example.com).\n\n![image](pic.png)\n\n- point\n\n1. one\n\n---\n\n```rust\nfn main() {}\n```\n\n| a | b |\n|---|---|\n| 1 | 2 |\n",
        );
    }

    #[test]
    fn nested_list_and_quote_are_built() {
        builds("- outer\n  - inner\n\n> quoted\n");
    }

    #[test]
    fn table_with_mixed_alignment_is_built() {
        builds("| a | b | c |\n|---|:-:|--:|\n| 1 |  | 3 |\n");
    }
}
