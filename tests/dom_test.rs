use linkedom_rust::Document;

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
