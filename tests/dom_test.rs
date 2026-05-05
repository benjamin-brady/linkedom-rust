use linkedom_rust::{Document, DomError};

#[test]
fn queries_and_mutates_wikipedia_like_html() {
    let mut doc = Document::parse(
        "<main><img class=\"thumb\" src=\"/a.png\"><p id=\"x\">Text</p><footer>Bye</footer></main>",
    );
    let imgs = doc.query_selector_all("img.thumb");
    assert_eq!(imgs.len(), 1);
    assert_eq!(doc.get_attribute(imgs[0], "src"), Some("/a.png".to_string()));
    let footer = doc.query_selector("footer").unwrap();
    doc.remove(footer).unwrap();
    assert!(!doc.serialize().contains("<footer>"));
    let p = doc.query_selector("#x").unwrap();
    doc.class_list_add(p, "selected").unwrap();
    assert!(doc.class_list_contains(p, "selected").unwrap());
}

#[test]
fn parses_and_serializes_html() {
    let doc = Document::parse("<main><h1>Hello</h1><img src=\"/x.png\"></main>");
    let html = doc.serialize();
    assert!(html.contains("<main>"));
    assert!(html.contains("<h1>Hello</h1>"));
    assert!(html.contains("<img src=\"/x.png\">"));
}

#[test]
fn serializes_text_escaping() {
    let doc = Document::parse("<p>a &amp; b &lt;c&gt;</p>");
    let html = doc.serialize();
    // html5ever decodes entities to raw chars; serializer must re-encode them
    assert!(html.contains("a &amp; b &lt;c&gt;"));
}

#[test]
fn serializes_attr_lt_escaped() {
    let doc = Document::parse("<img alt=\"a<b\">");
    let html = doc.serialize();
    assert!(html.contains("alt=\"a&lt;b\""), "< in attr values must be escaped, got: {html}");
}

#[test]
fn doctype_round_trip() {
    let doc = Document::parse("<!DOCTYPE html><html><head></head><body></body></html>");
    let html = doc.serialize();
    assert!(html.contains("<!DOCTYPE html>"), "doctype must survive round-trip, got: {html}");
}

#[test]
fn comment_round_trip() {
    let doc = Document::parse("<div><!-- my comment --></div>");
    let html = doc.serialize();
    assert!(html.contains("<!-- my comment -->"), "comment must survive round-trip, got: {html}");
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

// ── Task 4: selectors and DOM APIs ───────────────────────────────────────────

#[test]
fn selector_grouping_matches_multiple_tags() {
    let doc = Document::parse("<div><nav></nav><footer></footer><p></p></div>");
    let hits = doc.query_selector_all("footer, p");
    assert_eq!(hits.len(), 2, "got {hits:?}");
}

#[test]
fn selector_descendant_combinator() {
    let doc = Document::parse("<main><div><p>text</p></div></main>");
    let p = doc.query_selector("main p");
    assert!(p.is_some(), "descendant combinator should find <p>");
}

#[test]
fn selector_child_combinator_direct() {
    let doc = Document::parse("<main><img></main>");
    let hit = doc.query_selector("main > img");
    assert!(hit.is_some(), "direct child should match");
}

#[test]
fn selector_child_combinator_not_grandchild() {
    let doc = Document::parse("<main><div><img></div></main>");
    // img is NOT a direct child of main
    let hit = doc.query_selector("main > img");
    assert!(hit.is_none(), "img grandchild should not match `main > img`");
}

#[test]
fn get_and_set_attribute() {
    let mut doc = Document::parse("<p lang=\"en\">hi</p>");
    let p = doc.query_selector("p").unwrap();
    assert_eq!(doc.get_attribute(p, "lang"), Some("en".to_string()));
    doc.set_attribute(p, "lang", "fr").unwrap();
    assert_eq!(doc.get_attribute(p, "lang"), Some("fr".to_string()));
}

#[test]
fn remove_attribute_works() {
    let mut doc = Document::parse("<p lang=\"en\">hi</p>");
    let p = doc.query_selector("p").unwrap();
    doc.remove_attribute(p, "lang").unwrap();
    assert_eq!(doc.get_attribute(p, "lang"), None);
}

#[test]
fn set_attribute_on_non_element_returns_error() {
    let mut doc = Document::new();
    assert_eq!(doc.set_attribute(999, "x", "y"), Err(DomError::InvalidNode(999)));
}

#[test]
fn get_text_content_walks_descendants() {
    let doc = Document::parse("<div><p>Hello </p><span>world</span></div>");
    let div = doc.query_selector("div").unwrap();
    let text = doc.get_text_content(div).unwrap();
    assert_eq!(text, "Hello world");
}

#[test]
fn set_text_content_replaces_children() {
    let mut doc = Document::parse("<p>old</p>");
    let p = doc.query_selector("p").unwrap();
    doc.set_text_content(p, "new").unwrap();
    assert_eq!(doc.get_text_content(p).unwrap(), "new");
    assert!(doc.serialize().contains(">new<"));
}

#[test]
fn get_inner_html_returns_children() {
    let doc = Document::parse("<div><span>ok</span></div>");
    let div = doc.query_selector("div").unwrap();
    let html = doc.get_inner_html(div).unwrap();
    assert!(html.contains("<span>ok</span>"), "got: {html}");
}

#[test]
fn set_inner_html_replaces_children() {
    let mut doc = Document::parse("<div><p>old</p></div>");
    let div = doc.query_selector("div").unwrap();
    doc.set_inner_html(div, "<span>new</span>").unwrap();
    let html = doc.get_inner_html(div).unwrap();
    assert!(html.contains("<span>new</span>"), "got: {html}");
    assert!(!html.contains("<p>"), "old child should be gone, got: {html}");
}

#[test]
fn class_list_remove_works() {
    let mut doc = Document::parse("<p class=\"a b c\">hi</p>");
    let p = doc.query_selector("p").unwrap();
    doc.class_list_remove(p, "b").unwrap();
    assert!(!doc.class_list_contains(p, "b").unwrap());
    assert!(doc.class_list_contains(p, "a").unwrap());
    assert!(doc.class_list_contains(p, "c").unwrap());
}

#[test]
fn class_list_add_idempotent() {
    let mut doc = Document::parse("<p class=\"a\">hi</p>");
    let p = doc.query_selector("p").unwrap();
    doc.class_list_add(p, "a").unwrap();
    // Should still be one "a", not "a a"
    let attr = doc.get_attribute(p, "class").unwrap();
    assert_eq!(attr.split_whitespace().filter(|&c| c == "a").count(), 1);
}

#[test]
fn class_list_on_non_element_returns_error() {
    let doc = Document::new();
    assert!(matches!(
        doc.class_list_contains(999, "x"),
        Err(DomError::InvalidNode(999))
    ));
}

#[test]
fn create_text_node_can_be_appended() {
    let mut doc = Document::new();
    let div = doc.create_element("div");
    doc.append_child(doc.root_id(), div).unwrap();
    let txt = doc.create_text_node("hello");
    doc.append_child(div, txt).unwrap();
    assert_eq!(doc.get_text_content(div).unwrap(), "hello");
}

#[test]
fn attribute_selector_queries() {
    let doc = Document::parse("<img src=\"/a.png\"><img alt=\"logo\">");
    let with_src = doc.query_selector_all("img[src]");
    assert_eq!(with_src.len(), 1);
    let exact = doc.query_selector_all("img[src=\"/a.png\"]");
    assert_eq!(exact.len(), 1);
}

