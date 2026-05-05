/// All possible node kinds stored in the arena.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NodeKind {
    Document,
    // Scaffolding for Task 2+; variants unused until then.
    #[allow(dead_code)]
    DocumentType { name: String },
    #[allow(dead_code)]
    Element { tag: String },
    #[allow(dead_code)]
    Text { data: String },
    #[allow(dead_code)]
    Comment { data: String },
}

/// A single node in the arena.
#[derive(Debug, Clone)]
pub(crate) struct Node {
    // Scaffolding for Task 2+; fields unused until tree traversal is wired up.
    #[allow(dead_code)]
    pub(crate) kind: NodeKind,
    // Tree-link fields are scaffolding for Task 2+; unused until then.
    #[allow(dead_code)]
    pub(crate) parent: Option<u32>,
    #[allow(dead_code)]
    pub(crate) first_child: Option<u32>,
    #[allow(dead_code)]
    pub(crate) last_child: Option<u32>,
    #[allow(dead_code)]
    pub(crate) next_sibling: Option<u32>,
    #[allow(dead_code)]
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
