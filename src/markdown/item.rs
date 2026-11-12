use crate::markdown::Text;
use crate::markdown::uri::Uri;

pub use pulldown_cmark::{Alignment, HeadingLevel};

#[derive(Debug, Clone)]
pub enum Item {
    Heading(HeadingLevel, Text),
    Paragraph(Text),
    CodeBlock {
        language: Option<String>,
        code: String,
        lines: Vec<Text>,
    },
    List {
        start: Option<u64>,
        bullets: Vec<Bullet>,
    },
    Image {
        url: Uri,
        title: String,
        alt: Text,
    },
    Quote(Vec<Item>),
    Rule,
    Table {
        columns: Vec<Column>,
        rows: Vec<Row>,
    },
}

#[derive(Debug, Clone)]
pub struct Column {
    pub header: Vec<Item>,
    pub alignment: Alignment,
}

#[derive(Debug, Clone)]
pub struct Row {
    cells: Vec<Vec<Item>>,
}

impl Row {
    pub(super) fn new() -> Self {
        Self { cells: Vec::new() }
    }

    pub(super) fn push_cell(&mut self, cell: Vec<Item>) {
        self.cells.push(cell);
    }

    pub fn cells(&self) -> &[Vec<Item>] {
        &self.cells
    }
}

#[derive(Debug, Clone)]
pub enum Bullet {
    Point { items: Vec<Item> },
    Task { items: Vec<Item>, done: bool },
}

impl Bullet {
    pub(super) fn push(&mut self, item: Item) {
        let (Bullet::Point { items } | Bullet::Task { items, .. }) = self;

        items.push(item);
    }

    pub(super) fn mark_task(&mut self, done: bool) {
        let Bullet::Point { items } = self else {
            return;
        };

        *self = Bullet::Task {
            items: std::mem::take(items),
            done,
        };
    }

    pub(crate) fn items(&self) -> &[Item] {
        match self {
            Bullet::Point { items } | Bullet::Task { items, .. } => items,
        }
    }
}
