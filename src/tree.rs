use crate::arena::Arena;

/// Errors returned by public tree-mutation methods.
#[derive(Debug, PartialEq)]
pub enum DomError {
    /// The supplied node id does not exist in the arena.
    InvalidNode(u32),
    /// The document root may not be removed.
    CannotRemoveRoot,
    /// The document root may not be appended as a child of another node.
    CannotAppendRoot,
    /// The child node is already attached to a parent.
    AlreadyAttached,
}

impl std::fmt::Display for DomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomError::InvalidNode(id) => write!(f, "invalid node id: {id}"),
            DomError::CannotRemoveRoot => write!(f, "cannot remove the document root"),
            DomError::CannotAppendRoot => write!(f, "cannot append root as a child"),
            DomError::AlreadyAttached => write!(f, "child is already attached to a parent"),
        }
    }
}

impl std::error::Error for DomError {}

/// Append `child_id` as the last child of `parent_id`.
///
/// O(1): updates parent ↔ child and prev-sibling links only.
pub(crate) fn append_child(
    arena: &mut Arena,
    parent_id: u32,
    child_id: u32,
    root_id: u32,
) -> Result<(), DomError> {
    if arena.get(parent_id).is_none() {
        return Err(DomError::InvalidNode(parent_id));
    }
    if arena.get(child_id).is_none() {
        return Err(DomError::InvalidNode(child_id));
    }
    if child_id == root_id {
        return Err(DomError::CannotAppendRoot);
    }
    if arena.get(child_id).unwrap().parent.is_some() {
        return Err(DomError::AlreadyAttached);
    }

    // Read parent state before taking mutable refs.
    let last_child = arena.get(parent_id).unwrap().last_child;
    let first_child = arena.get(parent_id).unwrap().first_child;

    // Wire previous tail's forward link.
    if let Some(prev_id) = last_child {
        arena.get_mut(prev_id).unwrap().next_sibling = Some(child_id);
    }

    // Wire child's back-links.
    {
        let child = arena.get_mut(child_id).unwrap();
        child.parent = Some(parent_id);
        child.prev_sibling = last_child;
        child.next_sibling = None;
    }

    // Update parent's child pointers.
    {
        let parent = arena.get_mut(parent_id).unwrap();
        if first_child.is_none() {
            parent.first_child = Some(child_id);
        }
        parent.last_child = Some(child_id);
    }

    Ok(())
}

/// Detach `node_id` from its parent and siblings in O(1).
pub(crate) fn remove_node(
    arena: &mut Arena,
    node_id: u32,
    root_id: u32,
) -> Result<(), DomError> {
    if arena.get(node_id).is_none() {
        return Err(DomError::InvalidNode(node_id));
    }
    if node_id == root_id {
        return Err(DomError::CannotRemoveRoot);
    }

    // Snapshot links before any mutation.
    let parent = arena.get(node_id).unwrap().parent;
    let prev = arena.get(node_id).unwrap().prev_sibling;
    let next = arena.get(node_id).unwrap().next_sibling;

    // Stitch siblings together.
    if let Some(prev_id) = prev {
        arena.get_mut(prev_id).unwrap().next_sibling = next;
    }
    if let Some(next_id) = next {
        arena.get_mut(next_id).unwrap().prev_sibling = prev;
    }

    // Fix parent's first/last child pointers.
    if let Some(parent_id) = parent {
        let p = arena.get_mut(parent_id).unwrap();
        if p.first_child == Some(node_id) {
            p.first_child = next;
        }
        if p.last_child == Some(node_id) {
            p.last_child = prev;
        }
    }

    // Clear the detached node's tree links.
    let n = arena.get_mut(node_id).unwrap();
    n.parent = None;
    n.prev_sibling = None;
    n.next_sibling = None;

    Ok(())
}

/// Return an ordered list of direct children of `node_id`.
pub(crate) fn children(arena: &Arena, node_id: u32) -> Vec<u32> {
    let mut result = Vec::new();
    let Some(node) = arena.get(node_id) else {
        return result;
    };
    let mut cursor = node.first_child;
    while let Some(id) = cursor {
        result.push(id);
        cursor = arena.get(id).unwrap().next_sibling;
    }
    result
}

/// Return the parent id of `node_id`, or `None` if detached.
pub(crate) fn parent_of(arena: &Arena, node_id: u32) -> Option<u32> {
    arena.get(node_id)?.parent
}
