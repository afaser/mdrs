use std::cell::RefCell;
use std::sync::Arc;

use iced::advanced::text::{LineHeight, Shaping, Wrapping};
use iced::widget::markdown::Uri;
use iced::widget::text::Span;
use iced::{Element, Pixels};

use crate::config::{Config, Role};
use crate::selection::{
    Decoration, Generation, InlineConfig, InlineFlow, SelectableText, TextColors,
};
use crate::theme::FontMetricsBook;
use crate::theme::font::FontBook;
use crate::theme::styler::{RoleKey, SpanCache, SpanStyler};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum TextRole {
    Heading(usize),
    Paragraph,
    Code,
}

impl TextRole {
    fn key(self) -> RoleKey {
        match self {
            Self::Heading(index) => RoleKey::Heading(index),
            Self::Paragraph => RoleKey::Paragraph,
            Self::Code => RoleKey::Code,
        }
    }

    fn role(self, config: &Config) -> Role {
        match self {
            Self::Heading(index) => config.text.heading.role(index),
            Self::Paragraph => config.text.body.clone(),
            Self::Code => config.code_block.text.clone(),
        }
    }

    fn wrapping(self) -> Wrapping {
        match self {
            Self::Code => Wrapping::None,
            Self::Heading(_) | Self::Paragraph => Wrapping::default(),
        }
    }
}

struct PreparedText {
    spans: Arc<[Span<'static, Uri>]>,
    role: Role,
    font: iced::Font,
    colors: TextColors,
    decoration: Decoration,
    shaping: Shaping,
}

pub(super) struct ViewerText {
    config: Config,
    fonts: FontBook,
    styler: SpanStyler,
    inline: RefCell<SpanCache>,
    metrics: FontMetricsBook,
    generation: Generation,
}

impl ViewerText {
    pub(super) fn new(config: Config, fonts: FontBook) -> Self {
        Self {
            config,
            fonts,
            styler: SpanStyler::default(),
            inline: RefCell::default(),
            metrics: FontMetricsBook::default(),
            generation: Generation::default(),
        }
    }

    pub(super) fn config(&self) -> &Config {
        &self.config
    }

    pub(super) fn fonts(&self) -> &FontBook {
        &self.fonts
    }

    pub(super) fn set_generation(&mut self, generation: Generation) {
        self.generation = generation;
    }

    pub(super) fn clear(&self) {
        self.styler.clear();
        self.inline.borrow_mut().clear();
    }

    pub(super) fn selectable_text<'a, Message>(
        &self,
        role: TextRole,
        spans: Arc<[Span<'static, Uri>]>,
        size: Pixels,
    ) -> SelectableText<'a, Message>
    where
        Message: From<Uri> + From<Option<String>> + 'a,
    {
        let prepared = self.prepare(role, spans);

        self.selectable_from(role, prepared, size)
    }

    pub(super) fn flow_text<'a, Message>(
        &self,
        role: TextRole,
        spans: Arc<[Span<'static, Uri>]>,
        size: Pixels,
    ) -> Element<'a, Message, iced::Theme, iced::Renderer>
    where
        Message: From<Uri> + From<Option<String>> + 'a,
    {
        let prepared = self.prepare(role, spans);
        let has_code = prepared.spans.iter().any(|span| span.highlight.is_some());

        if role == TextRole::Code || !has_code {
            return self.selectable_from(role, prepared, size).into();
        }

        let space = self
            .metrics
            .metrics(prepared.font, size)
            .map(|metrics| metrics.space())
            .unwrap_or(size.0 * 0.25);
        let config = InlineConfig {
            size,
            font: prepared.font,
            line_height: LineHeight::Relative(prepared.role.line_height),
            shaping: prepared.shaping,
            space,
        };

        InlineFlow::new(prepared.spans, config, prepared.colors)
            .decoration(prepared.decoration)
            .generation(self.generation)
            .on_link_click(Message::from)
            .on_selection(Message::from)
            .into()
    }

    fn prepare(&self, role: TextRole, spans: Arc<[Span<'static, Uri>]>) -> PreparedText {
        let role_config = role.role(&self.config);
        let styled = self
            .styler
            .style(&spans, &role_config, role.key(), &self.fonts);
        let styled = self.style_links(styled);
        let styled = match role {
            TextRole::Code => styled,
            TextRole::Heading(_) | TextRole::Paragraph => self.inline_code(styled, role.key()),
        };
        let font = self
            .fonts
            .role_font(&role_config, role_config.weight, iced::font::Style::Normal)
            .unwrap_or_else(|| self.fonts.default_font(&role_config.fonts));
        let colors = TextColors {
            text: role_config.color.resolve(&self.config.palette),
            selection: self.config.selection.color.resolve(&self.config.palette),
        };
        let decoration = Decoration {
            underline_offset: self.config.selection.underline_offset,
            strikethrough_offset: self.config.selection.strikethrough_offset,
        };
        let shaping = match role_config.ligatures {
            true => Shaping::Advanced,
            false => Shaping::Auto,
        };

        PreparedText {
            spans: styled,
            role: role_config,
            font,
            colors,
            decoration,
            shaping,
        }
    }

    fn selectable_from<'a, Message>(
        &self,
        role: TextRole,
        prepared: PreparedText,
        size: Pixels,
    ) -> SelectableText<'a, Message>
    where
        Message: From<Uri> + From<Option<String>> + 'a,
    {
        SelectableText::new(prepared.spans, size, prepared.font, prepared.colors)
            .line_height(LineHeight::Relative(prepared.role.line_height))
            .wrapping(role.wrapping())
            .generation(self.generation)
            .decoration(prepared.decoration)
            .shaping(prepared.shaping)
            .on_link_click(Message::from)
            .on_selection(Message::from)
    }

    fn style_links(&self, spans: Arc<[Span<'static, Uri>]>) -> Arc<[Span<'static, Uri>]> {
        let link = &self.config.text.link;
        let configured_font = link.fonts.as_ref().and_then(|chain| {
            self.fonts.chain_font(
                chain,
                link.weight.unwrap_or(iced::font::Weight::Normal),
                iced::font::Style::Normal,
            )
        });
        let size = link.size.map(Pixels);
        let line_height = link.line_height.map(LineHeight::Relative);
        let letter_spacing = link.letter_spacing.map(iced::Pixels);
        let changed = link.underline
            || configured_font.is_some()
            || link.weight.is_some()
            || size.is_some()
            || line_height.is_some()
            || letter_spacing.is_some();

        if !changed {
            return spans;
        }

        spans
            .iter()
            .cloned()
            .map(|mut span| {
                if span.link.is_none() {
                    return span;
                }
                if link.underline {
                    span.underline = true;
                }
                self.apply_link_font(&mut span, configured_font, link.weight);
                if let Some(size) = size {
                    span.size = Some(size);
                }
                if let Some(line_height) = line_height {
                    span.line_height = Some(line_height);
                }
                if let Some(letter_spacing) = letter_spacing {
                    span.letter_spacing = Some(letter_spacing);
                }
                span
            })
            .collect::<Vec<_>>()
            .into()
    }

    fn apply_link_font(
        &self,
        span: &mut Span<'static, Uri>,
        configured: Option<iced::Font>,
        weight: Option<iced::font::Weight>,
    ) {
        if let Some(font) = configured {
            span.font = Some(font);
            return;
        }

        let Some(weight) = weight else {
            return;
        };
        let Some(mut font) = span.font else {
            return;
        };

        font.weight = weight;
        span.font = Some(font);
    }

    fn inline_code(
        &self,
        spans: Arc<[Span<'static, Uri>]>,
        key: RoleKey,
    ) -> Arc<[Span<'static, Uri>]> {
        if !spans.iter().any(|span| span.highlight.is_some()) {
            return spans;
        }

        let cache_key = (Arc::as_ptr(&spans) as *const () as usize, key);

        if let Some(cached) = self.inline.borrow().get(&cache_key) {
            return Arc::clone(cached);
        }

        let styled = self.style_inline_code(spans);
        self.inline
            .borrow_mut()
            .insert(cache_key, Arc::clone(&styled));
        styled
    }

    fn style_inline_code(&self, spans: Arc<[Span<'static, Uri>]>) -> Arc<[Span<'static, Uri>]> {
        let role = &self.config.text.inline_code.role;
        let font = self
            .fonts
            .role_font(role, role.weight, iced::font::Style::Normal);
        let size = Pixels(role.size);
        let line_height = LineHeight::Relative(role.line_height);
        let letter_spacing = (role.letter_spacing > 0.0).then_some(Pixels(role.letter_spacing));
        let color = role.color.resolve(&self.config.palette);

        spans
            .iter()
            .cloned()
            .map(|mut span| {
                if span.highlight.is_none() {
                    return span;
                }

                span.font = font.or(span.font);
                span.size = Some(size);
                span.line_height = Some(line_height);
                span.letter_spacing = letter_spacing;
                span.color = Some(color);
                span
            })
            .collect::<Vec<_>>()
            .into()
    }
}
