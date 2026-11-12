use pulldown_cmark::{CodeBlockKind, Event, Options, Parser as CmarkParser, Tag, TagEnd};

use crate::markdown::item::{Bullet, Column, Item, Row};
use crate::markdown::text::{Span, Text};
use crate::markdown::uri::Uri;

pub fn parse(source: &str) -> Vec<Item> {
    let mut parser = Parser::default();

    for (event, range) in CmarkParser::new_ext(source, options()).into_offset_iter() {
        parser.apply(event, range);
    }

    parser.items
}

fn options() -> Options {
    Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
        | Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS
        | Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
}

enum Scope {
    List(List),
    Quote(Vec<Item>),
    Table {
        alignment: Vec<pulldown_cmark::Alignment>,
        columns: Vec<Column>,
        rows: Vec<Row>,
        current: Vec<Item>,
    },
}

struct List {
    start: Option<u64>,
    bullets: Vec<Bullet>,
}

#[derive(Default)]
struct Parser {
    items: Vec<Item>,
    stack: Vec<Scope>,
    spans: Vec<Span>,
    code: String,
    code_language: Option<String>,
    code_lines: Vec<Text>,
    strong: bool,
    emphasis: bool,
    strikethrough: bool,
    metadata: bool,
    code_block: bool,
    link: Option<Uri>,
    image: Option<(Uri, String)>,
}

impl Parser {
    fn apply(&mut self, event: Event<'_>, source: std::ops::Range<usize>) {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(text) => self.text(text.into_string(), source),
            Event::Code(code) => self.code(code.into_string(), source),
            Event::SoftBreak => self.break_span(" ", source),
            Event::HardBreak => self.break_span("\n", source),
            Event::Rule if !self.metadata => self.produce(Item::Rule),
            Event::TaskListMarker(done) => self.task_marker(done),
            _ => {}
        }
    }

    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Strong if !self.metadata => self.strong = true,
            Tag::Emphasis if !self.metadata => self.emphasis = true,
            Tag::Strikethrough if !self.metadata => self.strikethrough = true,
            Tag::Link { dest_url, .. } if !self.metadata => {
                self.link = Some(dest_url.into_string());
            }
            Tag::Image {
                dest_url, title, ..
            } if !self.metadata => {
                self.image = Some((dest_url.into_string(), title.into_string()));
            }
            Tag::List(first_item) if !self.metadata => {
                self.flush_paragraph();
                self.stack.push(Scope::List(List {
                    start: first_item,
                    bullets: Vec::new(),
                }));
            }
            Tag::Item => {
                if let Some(Scope::List(list)) = self.stack.last_mut() {
                    list.bullets.push(Bullet::Point { items: Vec::new() });
                }
            }
            Tag::BlockQuote(_kind) if !self.metadata => {
                self.flush_paragraph();
                self.stack.push(Scope::Quote(Vec::new()));
            }
            Tag::CodeBlock(CodeBlockKind::Fenced(language)) if !self.metadata => {
                self.code_block = true;
                self.code_language = (!language.is_empty()).then(|| language.into_string());
                self.flush_paragraph();
            }
            Tag::MetadataBlock(_) => self.metadata = true,
            Tag::Table(alignment) => {
                self.stack.push(Scope::Table {
                    columns: Vec::with_capacity(alignment.len()),
                    alignment,
                    current: Vec::new(),
                    rows: Vec::new(),
                });
            }
            Tag::TableHead => self.strong = true,
            Tag::TableRow => {
                if let Some(Scope::Table { rows, .. }) = self.stack.last_mut() {
                    rows.push(Row::new());
                }
            }
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Heading(level) if !self.metadata && !self.spans.is_empty() => {
                let text = self.take_text();
                self.produce(Item::Heading(level, text));
            }
            TagEnd::Strong if !self.metadata => self.strong = false,
            TagEnd::Emphasis if !self.metadata => self.emphasis = false,
            TagEnd::Strikethrough if !self.metadata => self.strikethrough = false,
            TagEnd::Link if !self.metadata => self.link = None,
            TagEnd::Paragraph | TagEnd::Item if !self.metadata => self.flush_paragraph(),
            TagEnd::List(_) if !self.metadata => {
                let Some(Scope::List(list)) = self.stack.pop() else {
                    return;
                };

                self.produce(Item::List {
                    start: list.start,
                    bullets: list.bullets,
                });
            }
            TagEnd::BlockQuote(_kind) if !self.metadata => {
                let Some(Scope::Quote(quote)) = self.stack.pop() else {
                    return;
                };

                self.produce(Item::Quote(quote));
            }
            TagEnd::Image if !self.metadata => {
                let Some((url, title)) = self.image.take() else {
                    return;
                };

                let alt = self.take_text();
                self.produce(Item::Image { url, title, alt });
            }
            TagEnd::CodeBlock if !self.metadata => {
                self.code_block = false;

                let language = self.code_language.take();
                let code = std::mem::take(&mut self.code);
                let lines = std::mem::take(&mut self.code_lines);
                self.produce(Item::CodeBlock {
                    language,
                    code,
                    lines,
                });
            }
            TagEnd::MetadataBlock(_) => self.metadata = false,
            TagEnd::Table => {
                let Some(Scope::Table { columns, rows, .. }) = self.stack.pop() else {
                    return;
                };

                self.produce(Item::Table { columns, rows });
            }
            TagEnd::TableHead => self.strong = false,
            TagEnd::TableCell => self.table_cell(),
            _ => {}
        }
    }

    fn text(&mut self, text: String, source: std::ops::Range<usize>) {
        if self.metadata {
            return;
        }

        if self.code_block {
            self.code.push_str(&text);

            for (line, line_source) in code_line_sources(&text, source) {
                self.code_lines.push(Text::new(vec![Span {
                    text: line,
                    source: line_source,
                    strong: self.strong,
                    emphasis: self.emphasis,
                    strikethrough: self.strikethrough,
                    link: self.link.clone(),
                    code: false,
                }]));
            }

            return;
        }

        self.spans.push(Span {
            text,
            source,
            strong: self.strong,
            emphasis: self.emphasis,
            strikethrough: self.strikethrough,
            link: self.link.clone(),
            code: false,
        });
    }

    fn code(&mut self, code: String, source: std::ops::Range<usize>) {
        if self.metadata {
            return;
        }

        self.spans.push(Span {
            text: code,
            source,
            strong: self.strong,
            emphasis: self.emphasis,
            strikethrough: self.strikethrough,
            link: self.link.clone(),
            code: true,
        });
    }

    fn break_span(&mut self, text: &str, source: std::ops::Range<usize>) {
        if self.metadata {
            return;
        }

        self.spans.push(Span {
            text: text.to_owned(),
            source,
            strong: self.strong,
            emphasis: self.emphasis,
            strikethrough: self.strikethrough,
            link: self.link.clone(),
            code: false,
        });
    }

    fn task_marker(&mut self, done: bool) {
        let Some(Scope::List(list)) = self.stack.last_mut() else {
            return;
        };

        if let Some(bullet) = list.bullets.last_mut() {
            bullet.mark_task(done);
        }
    }

    fn table_cell(&mut self) {
        self.flush_paragraph();

        let Some(Scope::Table {
            alignment,
            columns,
            rows,
            current,
        }) = self.stack.last_mut()
        else {
            return;
        };

        let header = columns.len() < alignment.len();

        if header {
            let Some(alignment) = alignment.get(columns.len()).copied() else {
                return;
            };

            columns.push(Column {
                header: std::mem::take(current),
                alignment,
            });

            return;
        }

        if let Some(row) = rows.last_mut() {
            row.push_cell(std::mem::take(current));
        }
    }

    fn flush_paragraph(&mut self) {
        if self.spans.is_empty() {
            return;
        }

        let text = self.take_text();
        self.produce(Item::Paragraph(text));
    }

    fn take_text(&mut self) -> Text {
        Text::new(std::mem::take(&mut self.spans))
    }

    fn produce(&mut self, item: Item) {
        match self.stack.last_mut() {
            Some(Scope::List(list)) => {
                if let Some(bullet) = list.bullets.last_mut() {
                    bullet.push(item);
                }
            }
            Some(Scope::Quote(items)) => items.push(item),
            Some(Scope::Table { current, .. }) => current.push(item),
            None => self.items.push(item),
        }
    }
}

fn code_line_sources(
    text: &str,
    source: std::ops::Range<usize>,
) -> Vec<(String, std::ops::Range<usize>)> {
    let mut lines = Vec::new();
    let mut offset = 0;

    for segment in text.split_inclusive('\n') {
        let content = segment.strip_suffix('\n').unwrap_or(segment);
        let content = content.strip_suffix('\r').unwrap_or(content);
        let end = offset + content.len();

        lines.push((
            content.to_owned(),
            source.start + offset..source.start + end,
        ));
        offset += segment.len();
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = "# Heading\n\nParagraph with [link](https://example.com).\n\n![image](pic.png)\n\n> quote\n\n- point\n- point\n\n1. one\n2. two\n\n---\n\n```rust\nfn main() {}\n```\n\n| a | b |\n|---|---|\n| 1 | 2 |\n";

    fn bullets(items: &[Item]) -> &[Bullet] {
        items
            .iter()
            .find_map(|item| match item {
                Item::List { bullets, .. } => Some(bullets.as_slice()),
                _ => None,
            })
            .expect("list")
    }

    #[test]
    fn fixture_produces_every_item_variant() {
        let items = parse(FIXTURE);

        assert!(items.iter().any(|item| matches!(item, Item::Heading(..))));
        assert!(items.iter().any(|item| matches!(item, Item::Paragraph(_))));
        assert!(
            items
                .iter()
                .any(|item| matches!(item, Item::CodeBlock { .. }))
        );
        assert!(items.iter().any(|item| matches!(item, Item::List { .. })));
        assert!(items.iter().any(|item| matches!(item, Item::Image { .. })));
        assert!(items.iter().any(|item| matches!(item, Item::Quote(_))));
        assert!(items.iter().any(|item| matches!(item, Item::Rule)));
        assert!(items.iter().any(|item| matches!(item, Item::Table { .. })));
    }

    #[test]
    fn nested_list_lives_inside_bullet_items() {
        let items = parse("- outer\n  - inner\n");
        let outer = bullets(&items);

        assert_eq!(outer.len(), 1);
        assert!(
            outer[0]
                .items()
                .iter()
                .any(|item| matches!(item, Item::List { .. }))
        );
    }

    #[test]
    fn quote_contains_paragraph() {
        let items = parse("> quoted\n");
        let Item::Quote(contents) = &items[0] else {
            panic!("quote");
        };

        assert!(
            contents
                .iter()
                .any(|item| matches!(item, Item::Paragraph(_)))
        );
    }

    #[test]
    fn table_has_columns_rows_and_cells() {
        let items = parse("| a | b | c |\n|---|:-:|--:|\n| 1 | 2 | 3 |\n");
        let Item::Table { columns, rows } = &items[0] else {
            panic!("table");
        };

        assert_eq!(columns.len(), 3);
        assert_eq!(columns[1].alignment, pulldown_cmark::Alignment::Center);
        assert_eq!(columns[2].alignment, pulldown_cmark::Alignment::Right);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].cells().len(), 3);
    }

    #[test]
    fn task_list_marker_sets_done() {
        let items = parse("- [x] done\n- [ ] open\n");
        let list = bullets(&items);

        assert!(matches!(list[0], Bullet::Task { done: true, .. }));
        assert!(matches!(list[1], Bullet::Task { done: false, .. }));
    }

    #[test]
    fn metadata_block_is_skipped() {
        let items = parse("---\ntitle: hidden\n---\n\nvisible\n");

        assert_eq!(items.len(), 1);
        assert!(matches!(items[0], Item::Paragraph(_)));
    }
}
