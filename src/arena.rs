use crate::node::{Node, NodeKind};

/// Flat arena that owns all nodes, addressed by u32 index.
pub struct Arena {
    nodes: Vec<Node>,
}

impl Arena {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Allocate a new node and return its id.
    pub fn alloc(&mut self, kind: NodeKind) -> u32 {
        let id = self.nodes.len() as u32;
        self.nodes.push(Node::new(kind));
        id
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn get(&self, id: u32) -> Option<&Node> {
        self.nodes.get(id as usize)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut Node> {
        self.nodes.get_mut(id as usize)
    }
}
