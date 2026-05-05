use linkedom_rust::Document;

#[test]
fn creates_document_root() {
    let doc = Document::new();
    assert_eq!(doc.root_id(), 0);
    assert_eq!(doc.node_count(), 1);
}
