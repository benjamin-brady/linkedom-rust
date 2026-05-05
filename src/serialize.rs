use crate::{arena::Arena, node::NodeKind};

/// HTML void elements that must not have a closing tag.
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "keygen", "link", "meta",
    "param", "source", "track", "wbr",
];

fn is_void(tag: &str) -> bool {
    VOID_ELEMENTS.contains(&tag)
}

fn escape_text(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('"', "&quot;")
}

pub(crate) fn serialize_document(arena: &Arena, root_id: u32) -> String {
    let mut out = String::new();
    serialize_node(arena, root_id, &mut out);
    out
}

/// Serialize all children of `node_id` and return the HTML string (innerHTML).
/// Returns `None` if the node id is invalid.
pub(crate) fn serialize_inner_html(arena: &Arena, node_id: u32) -> Option<String> {
    let first_child = arena.get(node_id)?.first_child;
    let mut out = String::new();
    let mut cursor = first_child;
    while let Some(child_id) = cursor {
        serialize_node(arena, child_id, &mut out);
        cursor = arena.get(child_id).and_then(|n| n.next_sibling);
    }
    Some(out)
}

fn serialize_node(arena: &Arena, id: u32, out: &mut String) {
    let (kind, first_child) = {
        let node = arena.get(id).expect("serialize_node: invalid id");
        (node.kind.clone(), node.first_child)
    };

    match kind {
        NodeKind::Document => serialize_children(arena, first_child, out),
        NodeKind::DocumentType { name, .. } => {
            out.push_str("<!DOCTYPE ");
            out.push_str(&name);
            out.push('>');
        }
        NodeKind::Element { tag, attrs } => {
            out.push('<');
            out.push_str(&tag);
            for (k, v) in &attrs {
                out.push(' ');
                out.push_str(k);
                out.push_str("=\"");
                out.push_str(&escape_attr(v));
                out.push('"');
            }
            out.push('>');
            if !is_void(&tag) {
                serialize_children(arena, first_child, out);
                out.push_str("</");
                out.push_str(&tag);
                out.push('>');
            }
        }
        NodeKind::Text { data } => out.push_str(&escape_text(&data)),
        NodeKind::Comment { data } => {
            // Comment data is stored verbatim from the parser. The HTML spec
            // forbids "-->" inside a comment, so no further escaping is needed.
            out.push_str("<!--");
            out.push_str(&data);
            out.push_str("-->");
        }
    }
}

fn serialize_children(arena: &Arena, first_child: Option<u32>, out: &mut String) {
    let mut cursor = first_child;
    while let Some(child_id) = cursor {
        serialize_node(arena, child_id, out);
        cursor = arena
            .get(child_id)
            .expect("serialize_children: invalid child id")
            .next_sibling;
    }
}
