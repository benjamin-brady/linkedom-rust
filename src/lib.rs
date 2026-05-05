mod arena;
mod node;

use arena::Arena;
use node::NodeKind;

pub struct Document {
    arena: Arena,
    root: u32,
}

impl Document {
    #[must_use]
    pub fn new() -> Self {
        let mut arena = Arena::new();
        let root = arena.alloc(NodeKind::Document);
        Self { arena, root }
    }

    #[must_use]
    pub fn root_id(&self) -> u32 {
        self.root
    }

    #[must_use]
    pub fn node_count(&self) -> usize {
        self.arena.len()
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}
