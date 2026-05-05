//! WASM-facing API — thin wrapper around `Document` that converts `Result`/`Option`
//! to JS-friendly types and exposes a flat `#[wasm_bindgen]` interface.
//!
//! `WasmDocument` holds an owned `Document` and delegates every operation to it.
//! Error values from the core API are converted to `JsValue` strings so that
//! wasm-bindgen throws them as JavaScript `Error` objects.
//!
//! Numeric node ids (`u32`) flow through the JS boundary unchanged; the TypeScript
//! wrapper in `js/index.ts` hides them behind ergonomic classes.

use wasm_bindgen::prelude::*;

use crate::Document;

/// A DOM document exposed to JavaScript/WASM.
///
/// Construct one with [`WasmDocument::parse_html`].
#[wasm_bindgen]
pub struct WasmDocument {
    inner: Document,
}

#[wasm_bindgen]
impl WasmDocument {
    // ── Construction ─────────────────────────────────────────────────────────

    /// Parse an HTML string and return a new document.  Infallible: html5ever
    /// accepts any input without error.
    #[wasm_bindgen(js_name = parseHtml)]
    pub fn parse_html(html: &str) -> WasmDocument {
        WasmDocument { inner: Document::parse(html) }
    }

    /// Return the numeric id of the document root node.
    #[wasm_bindgen(js_name = rootId)]
    pub fn root_id(&self) -> u32 {
        self.inner.root_id()
    }

    // ── Serialization ────────────────────────────────────────────────────────

    /// Serialize the document back to an HTML string.
    pub fn serialize(&self) -> String {
        self.inner.serialize()
    }

    // ── Node creation ────────────────────────────────────────────────────────

    /// Allocate a detached element node and return its id.
    #[wasm_bindgen(js_name = createElement)]
    pub fn create_element(&mut self, tag: &str) -> u32 {
        self.inner.create_element(tag)
    }

    /// Allocate a detached text node and return its id.
    #[wasm_bindgen(js_name = createTextNode)]
    pub fn create_text_node(&mut self, text: &str) -> u32 {
        self.inner.create_text_node(text)
    }

    // ── Tree mutation ────────────────────────────────────────────────────────

    /// Append `child` as the last child of `parent`.
    ///
    /// Throws a JavaScript string error if either id is invalid or the child is
    /// already attached.
    #[wasm_bindgen(js_name = appendChild)]
    pub fn append_child(&mut self, parent: u32, child: u32) -> Result<(), JsValue> {
        self.inner.append_child(parent, child).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Detach `node` from its parent.  No-op if the node is already detached.
    ///
    /// Throws a JavaScript string error for invalid ids or an attempt to remove
    /// the root.
    pub fn remove(&mut self, node: u32) -> Result<(), JsValue> {
        self.inner.remove(node).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Return the ordered list of direct child ids of `node` as a `Uint32Array`.
    ///
    /// Returns an empty array for valid nodes with no children.
    /// Throws for invalid ids.
    pub fn children(&self, node: u32) -> Result<Vec<u32>, JsValue> {
        self.inner
            .children(node)
            .ok_or_else(|| JsValue::from_str(&format!("invalid node id: {node}")))
    }

    // ── Query ────────────────────────────────────────────────────────────────

    /// Return the id of the first element matching `selector`, or `undefined`
    /// if no element matches.
    #[wasm_bindgen(js_name = querySelector)]
    pub fn query_selector(&self, selector: &str) -> Option<u32> {
        self.inner.query_selector(selector)
    }

    /// Return all element ids matching `selector` in document order as a
    /// `Uint32Array`.
    #[wasm_bindgen(js_name = querySelectorAll)]
    pub fn query_selector_all(&self, selector: &str) -> Vec<u32> {
        self.inner.query_selector_all(selector)
    }

    // ── Attributes ───────────────────────────────────────────────────────────

    /// Get an attribute value.  Returns `undefined` if the attribute or node is
    /// absent.
    #[wasm_bindgen(js_name = getAttribute)]
    pub fn get_attribute(&self, node: u32, name: &str) -> Option<String> {
        self.inner.get_attribute(node, name)
    }

    /// Set an attribute on an element node, creating it if absent.  Throws for
    /// invalid ids or non-element nodes.
    #[wasm_bindgen(js_name = setAttribute)]
    pub fn set_attribute(&mut self, node: u32, name: &str, value: &str) -> Result<(), JsValue> {
        self.inner.set_attribute(node, name, value).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Remove an attribute from an element node.  No-op if the attribute is
    /// absent.  Throws for invalid ids or non-element nodes.
    #[wasm_bindgen(js_name = removeAttribute)]
    pub fn remove_attribute(&mut self, node: u32, name: &str) -> Result<(), JsValue> {
        self.inner
            .remove_attribute(node, name)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    // ── Text content ─────────────────────────────────────────────────────────

    /// Return the concatenated text of all text descendants, or `undefined` for
    /// unknown ids.
    #[wasm_bindgen(js_name = getTextContent)]
    pub fn get_text_content(&self, node: u32) -> Option<String> {
        self.inner.get_text_content(node)
    }

    /// Replace all children of `node` with a single text node.  Throws for
    /// invalid ids or non-element nodes.
    #[wasm_bindgen(js_name = setTextContent)]
    pub fn set_text_content(&mut self, node: u32, text: &str) -> Result<(), JsValue> {
        self.inner.set_text_content(node, text).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    // ── innerHTML ────────────────────────────────────────────────────────────

    /// Return the serialized inner HTML of `node`'s children, or `undefined`
    /// for unknown ids.
    #[wasm_bindgen(js_name = getInnerHtml)]
    pub fn get_inner_html(&self, node: u32) -> Option<String> {
        self.inner.get_inner_html(node)
    }

    /// Replace all children of `node` by parsing `html` as a fragment.  Throws
    /// for invalid ids or non-element nodes.
    #[wasm_bindgen(js_name = setInnerHtml)]
    pub fn set_inner_html(&mut self, node: u32, html: &str) -> Result<(), JsValue> {
        self.inner.set_inner_html(node, html).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    // ── classList ────────────────────────────────────────────────────────────

    /// Add a CSS class token to `node`'s `class` attribute.  Throws for invalid
    /// ids or non-element nodes.
    #[wasm_bindgen(js_name = classListAdd)]
    pub fn class_list_add(&mut self, node: u32, class: &str) -> Result<(), JsValue> {
        self.inner.class_list_add(node, class).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Remove a CSS class token from `node`'s `class` attribute.  Throws for
    /// invalid ids or non-element nodes.
    #[wasm_bindgen(js_name = classListRemove)]
    pub fn class_list_remove(&mut self, node: u32, class: &str) -> Result<(), JsValue> {
        self.inner.class_list_remove(node, class).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Return `true` if `node`'s `class` attribute contains the given token.
    /// Throws for invalid ids or non-element nodes.
    #[wasm_bindgen(js_name = classListContains)]
    pub fn class_list_contains(&self, node: u32, class: &str) -> Result<bool, JsValue> {
        self.inner
            .class_list_contains(node, class)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

// ── Native tests ─────────────────────────────────────────────────────────────
//
// `#[wasm_bindgen]` compiles for all targets; these tests exercise the wrapper
// delegation on the native (non-WASM) host without requiring the WASM target.
// Correctness of the underlying operations is covered by `tests/dom_test.rs`;
// these tests specifically verify that the WASM shim wires up correctly (i.e.
// `WasmDocument` is constructable and its calls reach the right `Document`
// methods).

#[cfg(test)]
mod tests {
    use super::WasmDocument;

    #[test]
    fn wasm_document_parse_and_serialize() {
        let doc = WasmDocument::parse_html("<h1>Hello WASM</h1>");
        let html = doc.serialize();
        assert!(html.contains("<h1>Hello WASM</h1>"), "serialize must round-trip: {html}");
    }

    #[test]
    fn wasm_document_root_id_is_zero() {
        let doc = WasmDocument::parse_html("<p>test</p>");
        assert_eq!(doc.root_id(), 0);
    }

    #[test]
    fn wasm_document_query_selector_and_attribute() {
        let doc = WasmDocument::parse_html(r#"<img class="thumb" src="/a.png">"#);
        let id = doc.query_selector("img.thumb");
        assert!(id.is_some(), "querySelector must find img.thumb");
        let src = doc.get_attribute(id.unwrap(), "src");
        assert_eq!(src.as_deref(), Some("/a.png"));
    }

    #[test]
    fn wasm_document_query_selector_all_returns_all() {
        let doc = WasmDocument::parse_html("<p>a</p><p>b</p><p>c</p>");
        let ids = doc.query_selector_all("p");
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn wasm_document_create_and_append() {
        let mut doc = WasmDocument::parse_html("<div id=\"root\"></div>");
        let root = doc.query_selector("#root").expect("root element");
        let span = doc.create_element("span");
        // In native (non-WASM) context, success-path calls to Result<(), JsValue>
        // methods work fine; only the error path calls wasm-bindgen's throw_val
        // which calls abort() natively.  We only test the happy path here.
        doc.append_child(root, span).unwrap();
        let html = doc.serialize();
        assert!(html.contains("<span>"), "appended span must appear in output");
    }

    #[test]
    fn wasm_document_remove_node() {
        let mut doc = WasmDocument::parse_html("<div><p>gone</p></div>");
        let p = doc.query_selector("p").expect("p element");
        doc.remove(p).unwrap();
        assert!(!doc.serialize().contains("<p>gone</p>"));
    }

    #[test]
    fn wasm_document_set_attribute() {
        let mut doc = WasmDocument::parse_html(r#"<img src="/old.png">"#);
        let img = doc.query_selector("img").expect("img");
        doc.set_attribute(img, "src", "/new.png").unwrap();
        assert_eq!(doc.get_attribute(img, "src").as_deref(), Some("/new.png"));
    }

    #[test]
    fn wasm_document_text_content() {
        let mut doc = WasmDocument::parse_html("<p>old text</p>");
        let p = doc.query_selector("p").expect("p");
        doc.set_text_content(p, "new text").unwrap();
        assert_eq!(doc.get_text_content(p).as_deref(), Some("new text"));
    }

    #[test]
    fn wasm_document_inner_html() {
        let mut doc = WasmDocument::parse_html("<div></div>");
        let div = doc.query_selector("div").expect("div");
        doc.set_inner_html(div, "<b>bold</b>").unwrap();
        let inner = doc.get_inner_html(div).expect("inner html");
        assert!(inner.contains("<b>bold</b>"), "inner html: {inner}");
    }

    #[test]
    fn wasm_document_class_list() {
        let mut doc = WasmDocument::parse_html("<p class=\"a\"></p>");
        let p = doc.query_selector("p").expect("p");
        doc.class_list_add(p, "b").unwrap();
        assert!(doc.class_list_contains(p, "b").unwrap());
        doc.class_list_remove(p, "a").unwrap();
        assert!(!doc.class_list_contains(p, "a").unwrap());
    }

    #[test]
    fn wasm_document_children_and_remove_attribute() {
        let mut doc = WasmDocument::parse_html(r#"<ul><li>a</li><li>b</li></ul>"#);
        let ul = doc.query_selector("ul").expect("ul");
        let kids = doc.children(ul).unwrap();
        assert_eq!(kids.len(), 2);
        // no-op: class attr is absent; no throw expected
        doc.remove_attribute(ul, "class").unwrap();
        let li = doc.query_selector("li").expect("li");
        doc.set_attribute(li, "data-x", "1").unwrap();
        doc.remove_attribute(li, "data-x").unwrap();
        assert_eq!(doc.get_attribute(li, "data-x"), None);
    }

    // NOTE: error-path tests for `Result<(), JsValue>` methods (e.g., invalid
    // node ids) are intentionally omitted from native Rust tests.  When
    // `#[wasm_bindgen]` methods return `Err(JsValue)`, wasm-bindgen internally
    // calls `throw_val` which invokes `libc::abort()` in non-WASM builds.
    // Error-path behaviour is exercised by the equivalent `Document` tests in
    // `tests/dom_test.rs`; the WASM shim is just a delegation layer.
}
