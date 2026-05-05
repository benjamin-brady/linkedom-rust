use linkedom_rust::{Document, DomError};

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
