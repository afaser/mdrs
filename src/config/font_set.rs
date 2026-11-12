use crate::config::FontChain;
use crate::config::node::Node;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontSet {
    pub regular: FontChain,
    pub bold: FontChain,
    pub italic: FontChain,
    pub bold_italic: FontChain,
}

impl FontSet {
    pub fn chain(&self, weight: iced::font::Weight, style: iced::font::Style) -> &FontChain {
        let bold = matches!(
            weight,
            iced::font::Weight::Semibold
                | iced::font::Weight::Bold
                | iced::font::Weight::ExtraBold
                | iced::font::Weight::Black
        );

        match (bold, style == iced::font::Style::Italic) {
            (true, true) => &self.bold_italic,
            (true, false) => &self.bold,
            (false, true) => &self.italic,
            (false, false) => &self.regular,
        }
    }

    fn from_node(base: Self, node: Node<'_>) -> Self {
        let regular = FontChain::from_node(base.regular, node.child("regular"));
        let bold = FontChain::from_node(base.bold, node.child("bold"));
        let italic = FontChain::from_node(base.italic, node.child("italic"));
        let bold_italic = FontChain::from_node(base.bold_italic, node.child("bold-italic"));

        Self {
            bold: fill(bold, &regular),
            italic: fill(italic, &regular),
            bold_italic: fill(bold_italic, &regular),
            regular,
        }
    }
}

fn fill(chain: FontChain, regular: &FontChain) -> FontChain {
    if chain.names().is_empty() {
        return regular.clone();
    }

    chain
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontSets {
    entries: Vec<(String, FontSet)>,
}

impl FontSets {
    pub fn get(&self, name: &str) -> Option<&FontSet> {
        self.entries
            .iter()
            .find(|(entry, _)| entry.eq_ignore_ascii_case(name))
            .map(|(_, set)| set)
    }

    pub(crate) fn from_node(base: Self, node: Option<Node<'_>>) -> Self {
        let Some(node) = node else {
            return base;
        };

        let mut sets = base;

        for child in node.children() {
            let Some(name) = child.name() else {
                continue;
            };

            let current = sets
                .get(name)
                .cloned()
                .unwrap_or_else(|| sets.default_set());
            let decoded = FontSet::from_node(current, child);

            match sets
                .entries
                .iter_mut()
                .find(|(entry, _)| entry.eq_ignore_ascii_case(name))
            {
                Some((_, set)) => *set = decoded,
                None => sets.entries.push((name.to_owned(), decoded)),
            }
        }

        sets
    }

    fn default_set(&self) -> FontSet {
        self.entries
            .first()
            .map(|(_, set)| FontSet {
                regular: set.regular.clone(),
                bold: FontChain::new(Vec::new()),
                italic: FontChain::new(Vec::new()),
                bold_italic: FontChain::new(Vec::new()),
            })
            .unwrap_or_else(|| FontSet {
                regular: FontChain::new(Vec::new()),
                bold: FontChain::new(Vec::new()),
                italic: FontChain::new(Vec::new()),
                bold_italic: FontChain::new(Vec::new()),
            })
    }
}

impl Default for FontSets {
    fn default() -> Self {
        let body = FontChain::new(vec!["latin".to_owned(), "cyrillic".to_owned()]);
        let code = FontChain::new(vec!["code".to_owned()]);
        let body_set = FontSet {
            regular: body.clone(),
            bold: body.clone(),
            italic: body.clone(),
            bold_italic: body,
        };
        let code_set = FontSet {
            regular: code.clone(),
            bold: code.clone(),
            italic: code.clone(),
            bold_italic: code,
        };

        Self {
            entries: vec![
                ("body".to_owned(), body_set.clone()),
                ("heading".to_owned(), body_set),
                ("code".to_owned(), code_set),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::document::Document;
    use crate::config::fonts::Fonts;

    #[test]
    fn default_font_sets_cover_body_heading_and_code() {
        let fonts = Fonts::default();
        let body = fonts.set("body").expect("body set");
        let code = fonts.set("code").expect("code set");

        assert_eq!(body.regular.names(), ["latin", "cyrillic"]);
        assert_eq!(code.regular.names(), ["code"]);
        assert_eq!(
            body.chain(iced::font::Weight::Bold, iced::font::Style::Italic)
                .names(),
            body.bold_italic.names()
        );
    }

    #[test]
    fn custom_font_set_is_decoded() {
        let document = Document::parse(
            r#"
fonts {
    sets {
        custom {
            regular "code"
            bold "latin" "cyrillic"
        }
    }
}
"#,
        )
        .expect("document");
        let fonts = Fonts::from_node(Fonts::default(), document.root().child("fonts"));
        let custom = fonts.set("custom").expect("custom set");

        assert_eq!(custom.regular.names(), ["code"]);
        assert_eq!(custom.bold.names(), ["latin", "cyrillic"]);
        assert_eq!(custom.italic.names(), ["code"]);
    }
}
