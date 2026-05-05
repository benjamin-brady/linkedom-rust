use crate::node::{Node, NodeKind};

/// Flat arena that owns all nodes, addressed by u32 index.
#[derive(Debug)]
pub(crate) struct Arena {
    nodes: Vec<Node>,
}

impl Arena {
    pub(crate) fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Allocate a new node and return its id.
    pub(crate) fn alloc(&mut self, kind: NodeKind) -> u32 {
        let id = u32::try_from(self.nodes.len())
            .expect("arena overflow: node count exceeds u32::MAX");
        self.nodes.push(Node::new(kind));
        id
    }

    pub(crate) fn len(&self) -> usize {
        self.nodes.len()
    }

    // Required to satisfy clippy::len_without_is_empty; not called in Task 2.
    #[allow(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub(crate) fn get(&self, id: u32) -> Option<&Node> {
        self.nodes.get(id as usize)
    }

    pub(crate) fn get_mut(&mut self, id: u32) -> Option<&mut Node> {
        self.nodes.get_mut(id as usize)
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}
