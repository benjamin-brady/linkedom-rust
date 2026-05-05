use linkedom_rust::{Document, DomError};

#[test]
fn parses_and_serializes_html() {
    let doc = Document::parse("<main><h1>Hello</h1><img src=\"/x.png\"></main>").unwrap();
    assert!(doc.serialize().contains("<main>"));
    assert!(doc.serialize().contains("<h1>Hello</h1>"));
    assert!(doc.serialize().contains("<img src=\"/x.png\">"));
}

#[test]
fn serializes_text_escaping() {
    let doc = Document::parse("<p>a &amp; b &lt;c&gt;</p>").unwrap();
    let html = doc.serialize();
    // html5ever decodes entities to raw chars; serializer must re-encode them
    assert!(html.contains("a &amp; b &lt;c&gt;"));
}

#[test]
fn append_child_invalid_parent_returns_error() {
    let mut doc = Document::new();
    let child = doc.create_element("div");
    assert_eq!(doc.append_child(999, child), Err(DomError::InvalidNode(999)));
}

#[test]
fn remove_invalid_id_returns_error() {
    let mut doc = Document::new();
    assert_eq!(doc.remove(999), Err(DomError::InvalidNode(999)));
}

#[test]
fn remove_middle_child_stitches_siblings() {
    let mut doc = Document::new();
    let first = doc.create_element("first");
    let middle = doc.create_element("middle");
    let third = doc.create_element("third");
    doc.append_child(doc.root_id(), first).unwrap();
    doc.append_child(doc.root_id(), middle).unwrap();
    doc.append_child(doc.root_id(), third).unwrap();
    doc.remove(middle).unwrap();
    assert_eq!(doc.children(doc.root_id()), Some(vec![first, third]));
}

#[test]
fn creates_document_root() {
    let doc = Document::new();
    assert_eq!(doc.root_id(), 0);
    assert_eq!(doc.node_count(), 1);
}

#[test]
fn default_matches_new_invariants() {
    let doc = Document::default();
    assert_eq!(doc.root_id(), 0);
    assert_eq!(doc.node_count(), 1);
}

#[test]
fn append_and_remove_are_linked_correctly() {
    let mut doc = Document::new();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    doc.append_child(0, div).unwrap();
    doc.append_child(0, span).unwrap();
    assert_eq!(doc.children(0), Some(vec![div, span]));
    doc.remove(span).unwrap();
    assert_eq!(doc.children(0), Some(vec![div]));
    assert_eq!(doc.parent(span), None);
}

#[test]
fn remove_root_returns_error() {
    let mut doc = Document::new();
    assert!(doc.remove(0).is_err());
}

#[test]
fn append_child_invalid_id_returns_error() {
    let mut doc = Document::new();
    assert!(doc.append_child(0, 999).is_err());
}

#[test]
fn append_root_as_child_returns_error() {
    let mut doc = Document::new();
    let div = doc.create_element("div");
    assert!(doc.append_child(div, 0).is_err());
}

#[test]
fn already_attached_returns_error() {
    let mut doc = Document::new();
    let div = doc.create_element("div");
    doc.append_child(0, div).unwrap();
    assert_eq!(doc.append_child(0, div), Err(DomError::AlreadyAttached));
}

#[test]
fn children_invalid_node_returns_none() {
    let doc = Document::new();
    assert_eq!(doc.children(999), None);
}

#[test]
fn children_valid_empty_node_returns_some_empty() {
    let doc = Document::new();
    assert_eq!(doc.children(doc.root_id()), Some(vec![]));
}

// Characterization test: remove on an already-detached (but valid) node is a no-op,
// matching browser `Node.remove()` semantics where calling remove on a node with no
// parent is a no-op that succeeds silently.
#[test]
fn remove_detached_valid_node_is_noop() {
    let mut doc = Document::new();
    let div = doc.create_element("div");
    // Never appended – already detached.
    assert_eq!(doc.remove(div), Ok(()));
    // Second call is also a no-op.
    assert_eq!(doc.remove(div), Ok(()));
}

// Characterization test: removing a parent from the document detaches only the parent
// from its own parent, leaving the parent's children intact as a detached subtree.
// A child of the removed parent still reports the removed parent as its parent.
#[test]
fn remove_parent_preserves_children_as_detached_subtree() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");
    doc.append_child(doc.root_id(), parent).unwrap();
    doc.append_child(parent, child).unwrap();

    doc.remove(parent).unwrap();

    // Parent is detached from the document root.
    assert_eq!(doc.parent(parent), None);
    assert_eq!(doc.children(doc.root_id()), Some(vec![]));

    // The subtree is preserved: child still knows its parent.
    assert_eq!(doc.parent(child), Some(parent));
    assert_eq!(doc.children(parent), Some(vec![child]));
}
