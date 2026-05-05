use crate::{arena::Arena, node::NodeKind, tree::DomError};

/// Add `class` to the element's `class` attribute if not already present.
pub(crate) fn class_list_add(arena: &mut Arena, node: u32, class: &str) -> Result<(), DomError> {
    let node_ref = arena.get_mut(node).ok_or(DomError::InvalidNode(node))?;
    match &mut node_ref.kind {
        NodeKind::Element { attrs, .. } => {
            if let Some((_, v)) = attrs.iter_mut().find(|(k, _)| k == "class") {
                if !v.split_whitespace().any(|c| c == class) {
                    if !v.is_empty() {
                        v.push(' ');
                    }
                    v.push_str(class);
                }
            } else {
                attrs.push(("class".to_string(), class.to_string()));
            }
            Ok(())
        }
        _ => Err(DomError::NotAnElement(node)),
    }
}

/// Remove `class` from the element's `class` attribute (no-op if absent).
pub(crate) fn class_list_remove(
    arena: &mut Arena,
    node: u32,
    class: &str,
) -> Result<(), DomError> {
    let node_ref = arena.get_mut(node).ok_or(DomError::InvalidNode(node))?;
    match &mut node_ref.kind {
        NodeKind::Element { attrs, .. } => {
            if let Some((_, v)) = attrs.iter_mut().find(|(k, _)| k == "class") {
                let new_val: String =
                    v.split_whitespace().filter(|&c| c != class).collect::<Vec<_>>().join(" ");
                *v = new_val;
            }
            Ok(())
        }
        _ => Err(DomError::NotAnElement(node)),
    }
}

/// Return `true` if the element's `class` attribute contains `class`.
pub(crate) fn class_list_contains(
    arena: &Arena,
    node: u32,
    class: &str,
) -> Result<bool, DomError> {
    let node_ref = arena.get(node).ok_or(DomError::InvalidNode(node))?;
    match &node_ref.kind {
        NodeKind::Element { attrs, .. } => {
            let has = attrs
                .iter()
                .find(|(k, _)| k == "class")
                .map(|(_, v)| v.split_whitespace().any(|c| c == class))
                .unwrap_or(false);
            Ok(has)
        }
        _ => Err(DomError::NotAnElement(node)),
    }
}
