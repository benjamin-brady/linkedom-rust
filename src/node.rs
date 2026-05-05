/// All possible node kinds stored in the arena.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NodeKind {
    Document,
    #[allow(dead_code)]
    DocumentType { name: String },
    Element { tag: String },
    #[allow(dead_code)]
    Text { data: String },
    #[allow(dead_code)]
    Comment { data: String },
}

/// A single node in the arena.
#[derive(Debug, Clone)]
pub(crate) struct Node {
    #[allow(dead_code)]
    pub(crate) kind: NodeKind,
    pub(crate) parent: Option<u32>,
    pub(crate) first_child: Option<u32>,
    pub(crate) last_child: Option<u32>,
    pub(crate) next_sibling: Option<u32>,
    pub(crate) prev_sibling: Option<u32>,
}

impl Node {
    pub(crate) fn new(kind: NodeKind) -> Self {
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
