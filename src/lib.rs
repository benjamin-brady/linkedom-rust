pub mod arena;
pub mod node;

use arena::Arena;
use node::NodeKind;

pub struct Document {
    arena: Arena,
    root: u32,
}

impl Document {
    pub fn new() -> Self {
        let mut arena = Arena::new();
        let root = arena.alloc(NodeKind::Document);
        Self { arena, root }
    }

    pub fn root_id(&self) -> u32 {
        self.root
    }

    pub fn node_count(&self) -> usize {
        self.arena.len()
    }
}
