mod arena;
mod class_list;
mod node;
mod parser;
mod selector;
mod serialize;
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

    /// Allocate a new text node and return its id. The node is detached until appended.
    #[must_use]
    pub fn create_text_node(&mut self, text: &str) -> u32 {
        self.arena.alloc(NodeKind::Text { data: text.to_string() })
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

    /// Parse an HTML string into a Document. html5ever is a forgiving HTML
    /// parser that never fails, so this is infallible.
    #[must_use]
    pub fn parse(html: &str) -> Document {
        parser::parse_html(html)
    }

    /// Serialize the document to an HTML string.
    #[must_use]
    pub fn serialize(&self) -> String {
        serialize::serialize_document(&self.arena, self.root)
    }

    // ── Query APIs ───────────────────────────────────────────────────────────

    /// Return the first element in document order that matches `selector`, or `None`.
    #[must_use]
    pub fn query_selector(&self, selector: &str) -> Option<u32> {
        selector::query_all(&self.arena, self.root, selector).into_iter().next()
    }

    /// Return all elements in document order that match `selector`.
    #[must_use]
    pub fn query_selector_all(&self, selector: &str) -> Vec<u32> {
        selector::query_all(&self.arena, self.root, selector)
    }

    // ── Attribute APIs ───────────────────────────────────────────────────────

    /// Get the value of an attribute on an element node.
    ///
    /// Returns `None` if the node does not exist, is not an element, or the attribute is absent.
    #[must_use]
    pub fn get_attribute(&self, node: u32, name: &str) -> Option<String> {
        let n = self.arena.get(node)?;
        match &n.kind {
            NodeKind::Element { attrs, .. } => {
                attrs.iter().find(|(k, _)| k == name).map(|(_, v)| v.clone())
            }
            _ => None,
        }
    }

    /// Set an attribute on an element node, creating it if absent.
    ///
    /// Returns `Err(DomError::InvalidNode)` for unknown ids and
    /// `Err(DomError::NotAnElement)` for non-element nodes.
    pub fn set_attribute(
        &mut self,
        node: u32,
        name: &str,
        value: &str,
    ) -> Result<(), DomError> {
        let n = self.arena.get_mut(node).ok_or(DomError::InvalidNode(node))?;
        match &mut n.kind {
            NodeKind::Element { attrs, .. } => {
                if let Some((_, v)) = attrs.iter_mut().find(|(k, _)| k == name) {
                    *v = value.to_string();
                } else {
                    attrs.push((name.to_string(), value.to_string()));
                }
                Ok(())
            }
            _ => Err(DomError::NotAnElement(node)),
        }
    }

    /// Remove an attribute from an element node (no-op if the attribute is absent).
    pub fn remove_attribute(&mut self, node: u32, name: &str) -> Result<(), DomError> {
        let n = self.arena.get_mut(node).ok_or(DomError::InvalidNode(node))?;
        match &mut n.kind {
            NodeKind::Element { attrs, .. } => {
                attrs.retain(|(k, _)| k != name);
                Ok(())
            }
            _ => Err(DomError::NotAnElement(node)),
        }
    }

    // ── Content APIs ─────────────────────────────────────────────────────────

    /// Return the concatenated text content of all text-node descendants.
    ///
    /// Returns `None` only if the node id does not exist in the arena.
    #[must_use]
    pub fn get_text_content(&self, node: u32) -> Option<String> {
        self.arena.get(node)?;
        let mut buf = String::new();
        collect_text(&self.arena, node, &mut buf);
        Some(buf)
    }

    /// Replace all children of `node` with a single text node containing `text`.
    pub fn set_text_content(&mut self, node: u32, text: &str) -> Result<(), DomError> {
        if self.arena.get(node).is_none() {
            return Err(DomError::InvalidNode(node));
        }
        let children = tree::children(&self.arena, node).unwrap_or_default();
        for child_id in children {
            tree::remove_node(&mut self.arena, child_id, self.root)?;
        }
        let text_id = self.arena.alloc(NodeKind::Text { data: text.to_string() });
        tree::append_child(&mut self.arena, node, text_id, self.root)
    }

    /// Return the serialized HTML of all children of `node` (innerHTML).
    ///
    /// Returns `None` if the node id does not exist in the arena.
    #[must_use]
    pub fn get_inner_html(&self, node: u32) -> Option<String> {
        serialize::serialize_inner_html(&self.arena, node)
    }

    /// Replace all children of `node` with the result of parsing `html` as a fragment.
    ///
    /// Implemented by wrapping `html` in a temporary `<div>` element, parsing the full
    /// document, finding that div, and copying its children into `node`.
    pub fn set_inner_html(&mut self, node: u32, html: &str) -> Result<(), DomError> {
        if self.arena.get(node).is_none() {
            return Err(DomError::InvalidNode(node));
        }
        // Remove existing children.
        let children = tree::children(&self.arena, node).unwrap_or_default();
        for child_id in children {
            tree::remove_node(&mut self.arena, child_id, self.root)?;
        }
        // Parse the fragment wrapped in a unique sentinel element.
        let wrapped = format!("<x-frag-sentinel>{html}</x-frag-sentinel>");
        let frag_doc = parser::parse_html(&wrapped);
        // Find the sentinel element in the parsed document.
        let sentinels = selector::query_all(&frag_doc.arena, frag_doc.root, "x-frag-sentinel");
        if let Some(&sentinel_id) = sentinels.first() {
            copy_subtree_children(&frag_doc, sentinel_id, &mut self.arena, node, self.root);
        }
        Ok(())
    }

    // ── classList APIs ───────────────────────────────────────────────────────

    /// Add `class` to the element's `class` attribute.
    pub fn class_list_add(&mut self, node: u32, class: &str) -> Result<(), DomError> {
        class_list::class_list_add(&mut self.arena, node, class)
    }

    /// Remove `class` from the element's `class` attribute.
    pub fn class_list_remove(&mut self, node: u32, class: &str) -> Result<(), DomError> {
        class_list::class_list_remove(&mut self.arena, node, class)
    }

    /// Return `true` if the element's `class` attribute contains `class`.
    pub fn class_list_contains(&self, node: u32, class: &str) -> Result<bool, DomError> {
        class_list::class_list_contains(&self.arena, node, class)
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

// ── Free helpers (crate-private) ─────────────────────────────────────────────

/// Recursively collect text content of all Text descendants into `buf`.
fn collect_text(arena: &Arena, node_id: u32, buf: &mut String) {
    let node = match arena.get(node_id) {
        Some(n) => n,
        None => return,
    };
    match &node.kind {
        NodeKind::Text { data } => buf.push_str(data),
        NodeKind::Document | NodeKind::Element { .. } => {
            let mut child = node.first_child;
            // `child` is Copy; the borrow of `node` (and thereby `arena`) ends here.
            // Re-entering `arena` in the loop is fine: both are shared borrows.
            while let Some(child_id) = child {
                collect_text(arena, child_id, buf);
                child = arena.get(child_id).and_then(|n| n.next_sibling);
            }
        }
        _ => {}
    }
}

/// Copy all children of `src_parent` from `src_doc`'s arena into `dst_arena` under `dst_parent`.
fn copy_subtree_children(
    src_doc: &Document,
    src_parent: u32,
    dst_arena: &mut Arena,
    dst_parent: u32,
    dst_root: u32,
) {
    let first_child = match src_doc.arena.get(src_parent) {
        Some(n) => n.first_child,
        None => return,
    };
    let mut cursor = first_child;
    while let Some(src_id) = cursor {
        let next = src_doc.arena.get(src_id).and_then(|n| n.next_sibling);
        copy_node_recursive(src_doc, src_id, dst_arena, dst_parent, dst_root);
        cursor = next;
    }
}

/// Deep-copy `src_id` from `src_doc` into `dst_arena` as a new child of `dst_parent`.
fn copy_node_recursive(
    src_doc: &Document,
    src_id: u32,
    dst_arena: &mut Arena,
    dst_parent: u32,
    dst_root: u32,
) {
    let (kind, first_child) = match src_doc.arena.get(src_id) {
        Some(n) => (n.kind.clone(), n.first_child),
        None => return,
    };
    let new_id = dst_arena.alloc(kind);
    let _ = tree::append_child(dst_arena, dst_parent, new_id, dst_root);

    let mut cursor = first_child;
    while let Some(child_id) = cursor {
        let next = src_doc.arena.get(child_id).and_then(|n| n.next_sibling);
        copy_node_recursive(src_doc, child_id, dst_arena, new_id, dst_root);
        cursor = next;
    }
}

