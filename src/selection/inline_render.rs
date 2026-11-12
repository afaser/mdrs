use iced::advanced::Renderer as _;
use iced::advanced::graphics::text::cosmic_text::Cursor;
use iced::advanced::renderer;
use iced::{Color, Point, Rectangle, Size, Vector};

use crate::selection::Selection;
use crate::selection::decoration::{self, Decoration, LineRule};
use crate::selection::inline_layout::Line;
use crate::selection::inline_state::InlineState;

pub(super) fn draw_chips(
    renderer: &mut iced::Renderer,
    origin: Point,
    lines: &[Line],
    state: &InlineState,
    clip: &Rectangle,
) {
    for line in lines {
        for item in &line.items {
            let atom = &state.layout().atoms()[item.atom];
            let Some(chip) = atom.chip() else {
                continue;
            };
            let top = line.baseline - item.ascent;
            let x = item.x;
            let width = chip.content_width() + chip.padding().x();
            let height = chip.content_ascent() + chip.content_descent() + chip.padding().y();
            let bounds = Rectangle::new(origin + Vector::new(x, top), Size::new(width, height));
            let Some(bounds) = bounds.intersection(clip) else {
                continue;
            };

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: chip.border(),
                    ..renderer::Quad::default()
                },
                chip.background(),
            );
        }
    }
}

pub(super) fn draw_decorations(
    renderer: &mut iced::Renderer,
    origin: Point,
    lines: &[Line],
    state: &InlineState,
    text: Color,
    decoration: Decoration,
    clip: &Rectangle,
) {
    for line in lines {
        for item in &line.items {
            let atom = &state.layout().atoms()[item.atom];
            let Some(body) = atom.text() else {
                continue;
            };
            let baseline = Vector::new(
                origin.x + item.x - item.offset,
                origin.y + line.baseline - item.ascent,
            );
            let run_baseline = body
                .paragraph()
                .buffer()
                .layout_runs()
                .next()
                .map(|run| run.line_y)
                .unwrap_or_default();
            let baseline = baseline + Vector::new(0.0, run_baseline);
            let Some(clip) = Rectangle::new(
                origin + Vector::new(item.x, line.baseline - item.ascent),
                Size::new(item.width, item.ascent + atom.descent()),
            )
            .intersection(clip) else {
                continue;
            };

            for style in body.styles() {
                let hovered = state
                    .hovered_link()
                    .is_some_and(|link| style.link.as_ref() == Some(link));

                if !style.underline && !style.strikethrough && !hovered {
                    continue;
                }

                let selection = Selection::new(
                    Cursor::new(0, style.range.start),
                    Cursor::new(0, style.range.end),
                );
                let regions = selection
                    .bounds(body.paragraph().buffer())
                    .into_iter()
                    .map(|bounds| {
                        Rectangle::new(
                            bounds.position()
                                + Vector::new(origin.x + item.x - item.offset, origin.y),
                            bounds.size(),
                        )
                    })
                    .filter_map(|bounds| bounds.intersection(&clip))
                    .collect::<Vec<_>>();
                let color = style.color.unwrap_or(text);

                if style.underline || hovered {
                    decoration::draw(
                        renderer,
                        &regions,
                        baseline,
                        style.size,
                        color,
                        LineRule::Underline,
                        decoration,
                    );
                }

                if style.strikethrough {
                    decoration::draw(
                        renderer,
                        &regions,
                        baseline,
                        style.size,
                        color,
                        LineRule::Strikethrough,
                        decoration,
                    );
                }
            }
        }
    }
}

pub(super) fn draw_text(
    renderer: &mut iced::Renderer,
    origin: Point,
    lines: &[Line],
    state: &InlineState,
    color: Color,
    clip: &Rectangle,
) {
    for line in lines {
        for item in &line.items {
            let atom = &state.layout().atoms()[item.atom];
            let Some(paragraph) = atom.paragraph() else {
                continue;
            };
            let x = item.x - item.offset + atom.content_offset();
            let y = line.baseline - atom.content_ascent();
            let height = atom.content_ascent() + atom.content_descent();
            let bounds = Rectangle::new(
                origin + Vector::new(x, y),
                Size::new(atom.content_width(), height),
            );
            let Some(clip) = Rectangle::new(
                origin + Vector::new(item.x, y),
                Size::new(item.width, height),
            )
            .intersection(clip) else {
                continue;
            };

            paragraph.draw(renderer, bounds, color, &clip);
        }
    }
}
