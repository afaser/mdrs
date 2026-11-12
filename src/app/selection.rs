use crate::selection::Generation;

#[derive(Debug, Default)]
pub(super) struct SelectionState {
    text: Option<String>,
    generation: Generation,
}

impl SelectionState {
    pub(super) fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    pub(super) fn generation(&self) -> Generation {
        self.generation
    }

    pub(super) fn set(&mut self, selection: String) {
        self.text = Some(selection);
    }

    pub(super) fn clear(&mut self) {
        self.text = None;
        self.generation.advance();
    }
}
