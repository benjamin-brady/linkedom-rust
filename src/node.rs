/// All possible node kinds stored in the arena.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Document,
    DocumentType { name: String },
    Element { tag: String },
    Text { data: String },
    Comment { data: String },
}

/// A single node in the arena.
#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub parent: Option<u32>,
    pub first_child: Option<u32>,
    pub last_child: Option<u32>,
    pub next_sibling: Option<u32>,
    pub prev_sibling: Option<u32>,
}

impl Node {
    pub fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            parent: None,
            first_child: None,
            last_child: None,
            next_sibling: None,
            prev_sibling: None,
        }
    }
}
