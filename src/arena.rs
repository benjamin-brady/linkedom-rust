use crate::node::{Node, NodeKind};

/// Flat arena that owns all nodes, addressed by u32 index.
pub(crate) struct Arena {
    nodes: Vec<Node>,
}

impl Arena {
    pub(crate) fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Allocate a new node and return its id.
    pub(crate) fn alloc(&mut self, kind: NodeKind) -> u32 {
        let id = self.nodes.len() as u32;
        self.nodes.push(Node::new(kind));
        id
    }

    pub(crate) fn len(&self) -> usize {
        self.nodes.len()
    }

    // Required to satisfy clippy::len_without_is_empty; not yet called in Task 1.
    #[allow(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}
