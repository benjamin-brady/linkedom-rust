mod arena;
mod node;
pub mod parser;
pub mod serialize;
pub(crate) mod tree;

use arena::Arena;
use node::NodeKind;
pub use tree::DomError;

#[derive(Debug)]
pub struct Document {
    pub(crate) arena: Arena,
    pub(crate) root: u32,
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
    #[must_use]
    pub fn create_element(&mut self, tag: &str) -> u32 {
        self.arena.alloc(NodeKind::Element { tag: tag.to_string(), attrs: Vec::new() })
    }

    /// Allocate any node kind and return its id.
    pub(crate) fn alloc_node(&mut self, kind: NodeKind) -> u32 {
        self.arena.alloc(kind)
    }

    /// Return an immutable reference to a node by id.
    pub(crate) fn get_node(&self, id: u32) -> Option<&node::Node> {
        self.arena.get(id)
    }

    /// Return a mutable reference to a node by id.
    pub(crate) fn get_node_mut(&mut self, id: u32) -> Option<&mut node::Node> {
        self.arena.get_mut(id)
    }

    /// Append `child` as the last child of `parent`. O(1).
    pub fn append_child(&mut self, parent: u32, child: u32) -> Result<(), DomError> {
        tree::append_child(&mut self.arena, parent, child, self.root)
    }

    /// Detach `node` from its parent and siblings. O(1).
    ///
    /// # No-op on detached nodes
    ///
    /// If `node` is valid but has no parent (already detached), returns `Ok(())` unchanged.
    /// This matches browser `Node.remove()` semantics.
    ///
    /// # Subtree preservation
    ///
    /// Only `node` is unlinked from its own parent. Its children remain attached to it,
    /// forming a self-consistent detached subtree. A child of the removed node will still
    /// report the removed node as its parent.
    pub fn remove(&mut self, node: u32) -> Result<(), DomError> {
        tree::remove_node(&mut self.arena, node, self.root)
    }

    /// Return the ordered list of direct children of `node`.
    ///
    /// Returns `None` if `node` does not exist in the arena.
    /// Returns `Some(vec![])` for a valid node with no children.
    #[must_use]
    pub fn children(&self, node: u32) -> Option<Vec<u32>> {
        tree::children(&self.arena, node)
    }

    /// Return the parent of `node`, or `None` if the node is detached, is the root, or does not
    /// exist in the arena.
    #[must_use]
    pub fn parent(&self, node: u32) -> Option<u32> {
        tree::parent_of(&self.arena, node)
    }

    /// Parse an HTML string into a Document.
    pub fn parse(html: &str) -> Result<Document, DomError> {
        parser::parse_html(html)
    }

    /// Serialize the document to an HTML string.
    #[must_use]
    pub fn serialize(&self) -> String {
        serialize::serialize_document(&self.arena, self.root)
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}
