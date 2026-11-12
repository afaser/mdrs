use crate::config::colors::ColorRef;
use crate::config::easing::Easing;
use crate::config::icon::IconSource;
use crate::config::node::Node;

#[derive(Debug, Clone, PartialEq)]
pub struct CopyButton {
    pub icon: IconSource,
    pub check_icon: IconSource,
    pub size: f32,
    pub button_size: f32,
    pub color: ColorRef,
    pub hover_color: ColorRef,
    pub background: ColorRef,
    pub hover_background: ColorRef,
    pub radius: f32,
    pub check_color: ColorRef,
    pub check_duration: f32,
    pub check_easing: Easing,
    pub hover_duration: f32,
    pub hover_easing: Easing,
}

impl Default for CopyButton {
    fn default() -> Self {
        Self {
            icon: IconSource::EmbeddedSvg(include_bytes!("../../assets/copy.svg")),
            check_icon: IconSource::EmbeddedSvg(include_bytes!("../../assets/check.svg")),
            size: 20.0,
            button_size: 28.0,
            color: ColorRef::Palette(crate::config::palette::PaletteKey::Text),
            hover_color: ColorRef::Palette(crate::config::palette::PaletteKey::Accent),
            background: ColorRef::Literal(iced::Color::TRANSPARENT),
            hover_background: ColorRef::Palette(crate::config::palette::PaletteKey::Surface),
            radius: 8.0,
            check_color: ColorRef::Palette(crate::config::palette::PaletteKey::Success),
            check_duration: 0.9,
            check_easing: Easing::EaseOut,
            hover_duration: 0.15,
            hover_easing: Easing::EaseOut,
        }
    }
}

impl CopyButton {
    pub(crate) fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        Self {
            icon: IconSource::from_node(node, "icon", "icon-path", base.icon),
            check_icon: IconSource::from_node(
                node,
                "check-icon",
                "check-icon-path",
                base.check_icon,
            ),
            size: node.number_child("size").unwrap_or(base.size),
            button_size: node.number_child("button-size").unwrap_or(base.button_size),
            color: node.color_child("color").unwrap_or(base.color),
            hover_color: node.color_child("hover-color").unwrap_or(base.hover_color),
            background: node.color_child("background").unwrap_or(base.background),
            hover_background: node
                .color_child("hover-background")
                .unwrap_or(base.hover_background),
            radius: node.number_child("radius").unwrap_or(base.radius),
            check_color: node.color_child("check-color").unwrap_or(base.check_color),
            check_duration: node
                .number_child("check-duration")
                .unwrap_or(base.check_duration),
            check_easing: easing(node, "check-easing", base.check_easing),
            hover_duration: node
                .number_child("hover-duration")
                .unwrap_or(base.hover_duration),
            hover_easing: easing(node, "hover-easing", base.hover_easing),
        }
    }
}

fn easing(node: Node<'_>, name: &str, base: Easing) -> Easing {
    let Some(node) = node.child(name) else {
        return base;
    };

    if let Some(value) = node.text().and_then(Easing::parse) {
        return value;
    }

    let numbers = node.numbers();

    if numbers.len() != 4 {
        return base;
    }

    Easing::from_parts(numbers[0], numbers[1], numbers[2], numbers[3]).unwrap_or(base)
}
