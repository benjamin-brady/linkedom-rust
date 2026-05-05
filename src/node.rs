// Node types define the DOM skeleton used in Task 2+; silence dead_code for now.
#![allow(dead_code)]

/// All possible node kinds stored in the arena.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NodeKind {
    Document,
    DocumentType { name: String },
    Element { tag: String },
    Text { data: String },
    Comment { data: String },
}

/// A single node in the arena.
#[derive(Debug, Clone)]
pub(crate) struct Node {
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
