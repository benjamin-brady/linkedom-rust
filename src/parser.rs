use std::borrow::Cow;
use std::cell::{Ref, RefCell};
use std::collections::HashMap;

use html5ever::{
    interface::tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeSink},
    parse_document,
    tendril::{StrTendril, TendrilSink},
    Attribute, QualName,
};

use crate::{node::NodeKind, Document, DomError};

/// TreeSink backed by our arena Document, using RefCell for interior mutability
/// (html5ever 0.39+ requires all TreeSink methods to take `&self`).
struct DomSink {
    document: RefCell<Document>,
    /// Maps node id → QualName for `elem_name` lookups.
    names: RefCell<HashMap<u32, QualName>>,
    /// Maps template element id → its content fragment id.
    template_contents: RefCell<HashMap<u32, u32>>,
}

impl DomSink {
    fn new() -> Self {
        Self {
            document: RefCell::new(Document::new()),
            names: RefCell::new(HashMap::new()),
            template_contents: RefCell::new(HashMap::new()),
        }
    }

    /// Append a node to a parent, detaching it first if already attached.
    fn append_node(&self, parent: u32, child: u32) {
        let has_parent = self
            .document
            .borrow()
            .get_node(child)
            .and_then(|n| n.parent)
            .is_some();
        if has_parent {
            let _ = self.document.borrow_mut().remove(child);
        }
        let _ = self.document.borrow_mut().append_child(parent, child);
    }

    /// Append text to parent, coalescing with a trailing text node when possible.
    fn append_text(&self, parent: u32, text: &str) {
        let last_child = self
            .document
            .borrow()
            .get_node(parent)
            .and_then(|n| n.last_child);

        if let Some(last_id) = last_child {
            let is_text = self
                .document
                .borrow()
                .get_node(last_id)
                .map(|n| matches!(n.kind, NodeKind::Text { .. }))
                .unwrap_or(false);
            if is_text {
                if let Some(node) = self.document.borrow_mut().get_node_mut(last_id) {
                    if let NodeKind::Text { data } = &mut node.kind {
                        data.push_str(text);
                        return;
                    }
                }
            }
        }

        let text_id = self
            .document
            .borrow_mut()
            .alloc_node(NodeKind::Text { data: text.to_owned() });
        let _ = self.document.borrow_mut().append_child(parent, text_id);
    }

    /// Insert `new_node` before `sibling` under `parent`, wiring all links.
    fn insert_before(&self, parent: u32, sibling: u32, new_node: u32) {
        let prev = self
            .document
            .borrow()
            .get_node(sibling)
            .and_then(|n| n.prev_sibling);

        {
            let mut doc = self.document.borrow_mut();
            if let Some(node) = doc.get_node_mut(new_node) {
                node.parent = Some(parent);
                node.prev_sibling = prev;
                node.next_sibling = Some(sibling);
            }
        }

        {
            let mut doc = self.document.borrow_mut();
            match prev {
                Some(prev_id) => {
                    if let Some(p) = doc.get_node_mut(prev_id) {
                        p.next_sibling = Some(new_node);
                    }
                }
                None => {
                    if let Some(p) = doc.get_node_mut(parent) {
                        p.first_child = Some(new_node);
                    }
                }
            }
        }

        {
            let mut doc = self.document.borrow_mut();
            if let Some(s) = doc.get_node_mut(sibling) {
                s.prev_sibling = Some(new_node);
            }
        }
    }

    /// Insert text before `sibling`, coalescing with an adjacent text node when possible.
    fn insert_text_before(&self, parent: u32, sibling: u32, text: &str) {
        let prev = self
            .document
            .borrow()
            .get_node(sibling)
            .and_then(|n| n.prev_sibling);

        if let Some(prev_id) = prev {
            let is_text = self
                .document
                .borrow()
                .get_node(prev_id)
                .map(|n| matches!(n.kind, NodeKind::Text { .. }))
                .unwrap_or(false);
            if is_text {
                if let Some(node) = self.document.borrow_mut().get_node_mut(prev_id) {
                    if let NodeKind::Text { data } = &mut node.kind {
                        data.push_str(text);
                        return;
                    }
                }
            }
        }

        let text_id = self
            .document
            .borrow_mut()
            .alloc_node(NodeKind::Text { data: text.to_owned() });
        self.insert_before(parent, sibling, text_id);
    }
}

impl TreeSink for DomSink {
    type Handle = u32;
    type Output = Document;
    type ElemName<'a>
        = Ref<'a, QualName>
    where
        Self: 'a;

    fn finish(self) -> Self::Output {
        self.document.into_inner()
    }

    fn parse_error(&self, _msg: Cow<'static, str>) {}

    fn get_document(&self) -> Self::Handle {
        self.document.borrow().root_id()
    }

    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        self.template_contents
            .borrow()
            .get(target)
            .copied()
            .unwrap_or(*target)
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x == y
    }

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> Self::ElemName<'a> {
        Ref::map(self.names.borrow(), |map| {
            map.get(target).expect("elem_name: unknown node id")
        })
    }

    fn create_element(
        &self,
        name: QualName,
        attrs: Vec<Attribute>,
        flags: ElementFlags,
    ) -> Self::Handle {
        let tag = name.local.as_ref().to_owned();
        let attrs_vec = attrs
            .iter()
            .map(|a| (a.name.local.as_ref().to_owned(), a.value.to_string()))
            .collect();
        let id = self
            .document
            .borrow_mut()
            .alloc_node(NodeKind::Element { tag, attrs: attrs_vec });
        self.names.borrow_mut().insert(id, name);

        if flags.template {
            let frag = self
                .document
                .borrow_mut()
                .alloc_node(NodeKind::Document);
            self.template_contents.borrow_mut().insert(id, frag);
        }

        id
    }

    fn create_comment(&self, text: StrTendril) -> Self::Handle {
        self.document
            .borrow_mut()
            .alloc_node(NodeKind::Comment { data: text.to_string() })
    }

    fn create_pi(&self, _target: StrTendril, _data: StrTendril) -> Self::Handle {
        self.document
            .borrow_mut()
            .alloc_node(NodeKind::Comment { data: String::new() })
    }

    fn append(&self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        match child {
            NodeOrText::AppendNode(id) => self.append_node(*parent, id),
            NodeOrText::AppendText(text) => self.append_text(*parent, &text),
        }
    }

    fn append_before_sibling(
        &self,
        sibling: &Self::Handle,
        new_node: NodeOrText<Self::Handle>,
    ) {
        let parent_id = match self
            .document
            .borrow()
            .get_node(*sibling)
            .and_then(|n| n.parent)
        {
            Some(p) => p,
            None => return,
        };

        match new_node {
            NodeOrText::AppendNode(id) => {
                let has_parent = self
                    .document
                    .borrow()
                    .get_node(id)
                    .and_then(|n| n.parent)
                    .is_some();
                if has_parent {
                    let _ = self.document.borrow_mut().remove(id);
                }
                self.insert_before(parent_id, *sibling, id);
            }
            NodeOrText::AppendText(text) => {
                self.insert_text_before(parent_id, *sibling, &text);
            }
        }
    }

    fn append_based_on_parent_node(
        &self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        let has_parent = self
            .document
            .borrow()
            .get_node(*element)
            .and_then(|n| n.parent)
            .is_some();
        if has_parent {
            self.append_before_sibling(element, child);
        } else {
            self.append(prev_element, child);
        }
    }

    fn append_doctype_to_document(
        &self,
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    ) {
        let doctype_id = self.document.borrow_mut().alloc_node(NodeKind::DocumentType {
            name: name.to_string(),
            public_id: public_id.to_string(),
            system_id: system_id.to_string(),
        });
        let root = self.document.borrow().root_id();
        let _ = self.document.borrow_mut().append_child(root, doctype_id);
    }

    fn set_quirks_mode(&self, _mode: QuirksMode) {}

    fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<Attribute>) {
        let mut doc = self.document.borrow_mut();
        if let Some(node) = doc.get_node_mut(*target) {
            if let NodeKind::Element { attrs: existing, .. } = &mut node.kind {
                for attr in attrs {
                    let name = attr.name.local.as_ref().to_owned();
                    if !existing.iter().any(|(k, _)| k == &name) {
                        existing.push((name, attr.value.to_string()));
                    }
                }
            }
        }
    }

    fn remove_from_parent(&self, target: &Self::Handle) {
        let _ = self.document.borrow_mut().remove(*target);
    }

    fn reparent_children(&self, node: &Self::Handle, new_parent: &Self::Handle) {
        let children: Vec<u32> = {
            let doc = self.document.borrow();
            let mut result = Vec::new();
            let mut cursor = doc.get_node(*node).and_then(|n| n.first_child);
            while let Some(child_id) = cursor {
                result.push(child_id);
                cursor = doc.get_node(child_id).and_then(|n| n.next_sibling);
            }
            result
        };
        for child in children {
            let _ = self.document.borrow_mut().remove(child);
            let _ = self.document.borrow_mut().append_child(*new_parent, child);
        }
    }
}

pub(crate) fn parse_html(html: &str) -> Result<Document, DomError> {
    let sink = DomSink::new();
    let doc = parse_document(sink, Default::default()).one(html);
    Ok(doc)
}
