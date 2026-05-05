mod arena;
mod node;
pub mod tree;

use arena::Arena;
use node::NodeKind;
pub use tree::DomError;

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

    /// Allocate a new element node and return its id. The node is detached until appended.
    pub fn create_element(&mut self, tag: &str) -> u32 {
        self.arena.alloc(NodeKind::Element { tag: tag.to_string() })
    }

    /// Append `child` as the last child of `parent`. O(1).
    pub fn append_child(&mut self, parent: u32, child: u32) -> Result<(), DomError> {
        tree::append_child(&mut self.arena, parent, child, self.root)
    }

    /// Detach `node` from its parent and siblings. O(1).
    pub fn remove(&mut self, node: u32) -> Result<(), DomError> {
        tree::remove_node(&mut self.arena, node, self.root)
    }

    /// Return the ordered list of direct children of `node`.
    #[must_use]
    pub fn children(&self, node: u32) -> Vec<u32> {
        tree::children(&self.arena, node)
    }

    /// Return the parent of `node`, or `None` if detached or root.
    #[must_use]
    pub fn parent(&self, node: u32) -> Option<u32> {
        tree::parent_of(&self.arena, node)
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}
