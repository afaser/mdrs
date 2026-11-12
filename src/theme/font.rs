use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Once;

use iced::Font;
use iced::font::{Family, Stretch, Style, Weight};

use crate::config::{Face, FontChain, Fonts, NamedFont, Role, Script};

const FAMILY: &str = "JetBrainsMono Nerd Font";

const FONTCONFIG: &str = "<?xml version=\"1.0\"?><!DOCTYPE fontconfig SYSTEM \"fonts.dtd\"><fontconfig><dir>/nonexistent/mdrs-fonts</dir></fontconfig>";

static ISOLATE: Once = Once::new();

pub fn isolate_system_fonts() {
    ISOLATE.call_once(|| {
        if std::env::var_os("FONTCONFIG_FILE").is_some() {
            return;
        }

        let configuration = std::env::temp_dir().join("mdrs-fonts.conf");

        if std::fs::write(&configuration, FONTCONFIG).is_err() {
            return;
        }

        unsafe {
            std::env::set_var("FONTCONFIG_FILE", configuration);
        }
    });
}

#[derive(Debug, Clone)]
pub struct FontBook {
    fonts: Fonts,
    families: BTreeMap<String, &'static str>,
}

impl FontBook {
    pub fn new(fonts: &Fonts) -> Self {
        let mut families: BTreeMap<String, &'static str> = BTreeMap::new();

        for font in fonts.iter() {
            if families.contains_key(&font.family) {
                continue;
            }

            let family = Box::leak(font.family.clone().into_boxed_str());
            families.insert(font.family.clone(), family);
        }

        Self {
            fonts: fonts.clone(),
            families,
        }
    }

    pub fn default_font(&self, chain: &FontChain) -> Font {
        self.chain_font(chain, Weight::Normal, Style::Normal)
            .unwrap_or_else(|| Font::with_name(FAMILY))
    }

    pub fn chain_font(&self, chain: &FontChain, weight: Weight, style: Style) -> Option<Font> {
        let name = chain.first()?;
        let font = self.fonts.named(name)?;

        self.named_font(font, weight, style)
    }

    pub fn script_font(
        &self,
        chain: &FontChain,
        script: Script,
        weight: Weight,
        style: Style,
    ) -> Option<Font> {
        let named = self.script_named(chain, script)?;

        self.named_font(named, weight, style)
    }

    pub fn role_font(&self, role: &Role, weight: Weight, style: Style) -> Option<Font> {
        self.chain_font(role.chain(&self.fonts, weight, style), weight, style)
    }

    pub fn role_script_font(
        &self,
        role: &Role,
        script: Script,
        weight: Weight,
        style: Style,
    ) -> Option<Font> {
        self.script_font(
            role.chain(&self.fonts, weight, style),
            script,
            weight,
            style,
        )
    }

    pub fn script_named(&self, chain: &FontChain, script: Script) -> Option<&NamedFont> {
        self.chain_named(chain)?;

        match script {
            Script::Common => self.chain_named(chain),
            Script::Latin | Script::Cyrillic => chain
                .names()
                .iter()
                .filter_map(|name| self.fonts.named(name))
                .find(|font| font.script == script || font.script == Script::Any)
                .or_else(|| self.chain_named(chain)),
            Script::Any => self.chain_named(chain),
        }
    }

    pub fn chain_named(&self, chain: &FontChain) -> Option<&NamedFont> {
        chain.first().and_then(|name| self.fonts.named(name))
    }

    pub fn named_font(&self, font: &NamedFont, weight: Weight, style: Style) -> Option<Font> {
        let family = self.families.get(&font.family)?;

        Some(Font {
            family: Family::Name(family),
            weight,
            style,
            stretch: Stretch::Normal,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LoadedFonts {
    pub book: FontBook,
    pub bytes: Vec<Cow<'static, [u8]>>,
}

impl LoadedFonts {
    pub fn load(fonts: &Fonts) -> Self {
        let book = FontBook::new(fonts);
        let mut bytes = Vec::new();

        for font in fonts.iter() {
            for face in [
                &font.faces.regular,
                &font.faces.bold,
                &font.faces.italic,
                &font.faces.bold_italic,
            ] {
                load_face(face, &mut bytes);
            }
        }

        Self { book, bytes }
    }
}

#[cfg(test)]
pub(crate) fn load_embedded_fonts() {
    static LOAD: Once = Once::new();

    isolate_system_fonts();

    LOAD.call_once(|| {
        let loaded = LoadedFonts::load(&Fonts::default());
        let mut font_system = iced::advanced::graphics::text::font_system()
            .write()
            .expect("font system");

        for bytes in loaded.bytes {
            font_system
                .raw()
                .db_mut()
                .load_font_data(bytes.into_owned());
        }
    });
}

fn load_face(face: &Face, bytes: &mut Vec<Cow<'static, [u8]>>) {
    if let Some(data) = face.source.bytes() {
        bytes.push(Cow::Borrowed(data));
        return;
    }

    let Some(path) = face.source.path() else {
        return;
    };

    match std::fs::read(path) {
        Ok(data) => bytes.push(Cow::Owned(data)),
        Err(error) => log::warn!("font: {}: {error}", path.display()),
    }
}
