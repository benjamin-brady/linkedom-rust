use crate::{arena::Arena, node::NodeKind};

// ── Data structures ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AttrSelector {
    pub(crate) name: String,
    pub(crate) value: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SimpleSelector {
    /// `None` means universal (`*`).
    pub(crate) tag: Option<String>,
    pub(crate) id: Option<String>,
    pub(crate) classes: Vec<String>,
    pub(crate) attrs: Vec<AttrSelector>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Combinator {
    /// Whitespace – any ancestor.
    Descendant,
    /// `>` – direct parent only.
    Child,
}

/// One step in a complex selector.
/// `combinator` describes how this step relates to the *previous* step (`None` for leftmost).
#[derive(Debug, Clone)]
pub(crate) struct ComplexStep {
    pub(crate) simple: SimpleSelector,
    pub(crate) combinator: Option<Combinator>,
}

/// A chained sequence of simple selectors joined by combinators.
#[derive(Debug, Clone)]
pub(crate) struct ComplexSelector {
    pub(crate) steps: Vec<ComplexStep>,
}

/// Comma-separated group of complex selectors.
pub(crate) type SelectorGroup = Vec<ComplexSelector>;

// ── Parser ───────────────────────────────────────────────────────────────────

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        Self { chars: input.chars().collect(), pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn consume(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied()?;
        self.pos += 1;
        Some(c)
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(' ') | Some('\t') | Some('\n') | Some('\r')) {
            self.consume();
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn parse_ident(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                s.push(c);
                self.consume();
            } else {
                break;
            }
        }
        s
    }

    fn parse_attr_name(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c == '=' || c == ']' || c == ' ' || c == '\t' {
                break;
            }
            s.push(c);
            self.consume();
        }
        s
    }

    fn parse_attr_value(&mut self) -> String {
        match self.peek() {
            Some('"') => {
                self.consume();
                let mut s = String::new();
                while let Some(c) = self.peek() {
                    if c == '"' {
                        self.consume();
                        break;
                    }
                    s.push(c);
                    self.consume();
                }
                s
            }
            Some('\'') => {
                self.consume();
                let mut s = String::new();
                while let Some(c) = self.peek() {
                    if c == '\'' {
                        self.consume();
                        break;
                    }
                    s.push(c);
                    self.consume();
                }
                s
            }
            _ => {
                let mut s = String::new();
                while let Some(c) = self.peek() {
                    if c == ']' || c == ' ' || c == '\t' {
                        break;
                    }
                    s.push(c);
                    self.consume();
                }
                s
            }
        }
    }

    /// Parse a single simple selector (tag/id/classes/attrs). Returns `None` if
    /// the current position has nothing that looks like a selector.
    fn parse_simple_selector(&mut self) -> Option<SimpleSelector> {
        let mut tag = None;
        let mut id = None;
        let mut classes = Vec::new();
        let mut attrs = Vec::new();

        // Optional tag or universal `*`
        match self.peek() {
            Some('*') => {
                self.consume();
            }
            Some(c) if c.is_alphabetic() => {
                let t = self.parse_ident();
                if !t.is_empty() {
                    tag = Some(t);
                }
            }
            _ => {}
        }

        // Qualifiers: #id .class [attr]
        loop {
            match self.peek() {
                Some('#') => {
                    self.consume();
                    let i = self.parse_ident();
                    if !i.is_empty() {
                        id = Some(i);
                    }
                }
                Some('.') => {
                    self.consume();
                    let c = self.parse_ident();
                    if !c.is_empty() {
                        classes.push(c);
                    }
                }
                Some('[') => {
                    self.consume();
                    self.skip_whitespace();
                    let name = self.parse_attr_name();
                    self.skip_whitespace();
                    let value = if self.peek() == Some('=') {
                        self.consume();
                        self.skip_whitespace();
                        Some(self.parse_attr_value())
                    } else {
                        None
                    };
                    self.skip_whitespace();
                    if self.peek() == Some(']') {
                        self.consume();
                    }
                    if !name.is_empty() {
                        attrs.push(AttrSelector { name, value });
                    }
                }
                _ => break,
            }
        }

        if tag.is_none() && id.is_none() && classes.is_empty() && attrs.is_empty() {
            None
        } else {
            Some(SimpleSelector { tag, id, classes, attrs })
        }
    }

    /// Parse a complex selector (one or more simple selectors joined by combinators).
    /// Stops at EOF or `,`.
    fn parse_complex_selector(&mut self) -> Option<ComplexSelector> {
        let mut steps = Vec::new();

        self.skip_whitespace();
        let first = self.parse_simple_selector()?;
        steps.push(ComplexStep { simple: first, combinator: None });

        loop {
            if self.is_eof() || self.peek() == Some(',') {
                break;
            }

            // Leading whitespace is significant: it may be the descendant combinator.
            let had_whitespace =
                matches!(self.peek(), Some(' ') | Some('\t') | Some('\n') | Some('\r'));
            self.skip_whitespace();

            if self.is_eof() || self.peek() == Some(',') {
                break;
            }

            let combinator = if self.peek() == Some('>') {
                self.consume();
                self.skip_whitespace();
                Combinator::Child
            } else if had_whitespace {
                Combinator::Descendant
            } else {
                break; // adjacent tokens without space — shouldn't happen in valid CSS
            };

            let simple = match self.parse_simple_selector() {
                Some(s) => s,
                None => break,
            };

            steps.push(ComplexStep { simple, combinator: Some(combinator) });
        }

        if steps.is_empty() { None } else { Some(ComplexSelector { steps }) }
    }
}

/// Parse a CSS selector group (comma-separated complex selectors).
pub(crate) fn parse_selector_group(input: &str) -> SelectorGroup {
    let mut parser = Parser::new(input);
    let mut result = Vec::new();

    loop {
        parser.skip_whitespace();
        if parser.is_eof() {
            break;
        }
        if let Some(complex) = parser.parse_complex_selector() {
            result.push(complex);
        }
        parser.skip_whitespace();
        if parser.peek() == Some(',') {
            parser.consume();
        } else {
            break;
        }
    }

    result
}

// ── Matching ─────────────────────────────────────────────────────────────────

fn matches_simple(arena: &Arena, node_id: u32, simple: &SimpleSelector) -> bool {
    let node = match arena.get(node_id) {
        Some(n) => n,
        None => return false,
    };

    let (tag, attrs) = match &node.kind {
        NodeKind::Element { tag, attrs } => (tag, attrs),
        _ => return false,
    };

    if let Some(t) = &simple.tag {
        if t != tag {
            return false;
        }
    }

    if let Some(id) = &simple.id {
        if !attrs.iter().any(|(k, v)| k == "id" && v == id) {
            return false;
        }
    }

    for cls in &simple.classes {
        let class_attr =
            attrs.iter().find(|(k, _)| k == "class").map(|(_, v)| v.as_str()).unwrap_or("");
        if !class_attr.split_whitespace().any(|c| c == cls.as_str()) {
            return false;
        }
    }

    for attr_sel in &simple.attrs {
        match &attr_sel.value {
            None => {
                if !attrs.iter().any(|(k, _)| k == &attr_sel.name) {
                    return false;
                }
            }
            Some(expected) => {
                if !attrs.iter().any(|(k, v)| k == &attr_sel.name && v == expected) {
                    return false;
                }
            }
        }
    }

    true
}

/// Match a node against a complex selector, walking ancestors as needed.
/// `step_idx` is the index of the rightmost step we're currently trying to satisfy.
fn matches_from_right(
    arena: &Arena,
    node_id: u32,
    steps: &[ComplexStep],
    step_idx: usize,
) -> bool {
    let step = &steps[step_idx];

    if !matches_simple(arena, node_id, &step.simple) {
        return false;
    }

    if step_idx == 0 {
        return true;
    }

    let combinator = step.combinator.as_ref().expect("non-first step always has a combinator");

    match combinator {
        Combinator::Child => {
            let parent_id = match arena.get(node_id).and_then(|n| n.parent) {
                Some(p) => p,
                None => return false,
            };
            matches_from_right(arena, parent_id, steps, step_idx - 1)
        }
        Combinator::Descendant => {
            let mut ancestor = arena.get(node_id).and_then(|n| n.parent);
            while let Some(anc_id) = ancestor {
                if matches_from_right(arena, anc_id, steps, step_idx - 1) {
                    return true;
                }
                ancestor = arena.get(anc_id).and_then(|n| n.parent);
            }
            false
        }
    }
}

pub(crate) fn element_matches(arena: &Arena, node_id: u32, group: &SelectorGroup) -> bool {
    group.iter().any(|complex| {
        let steps = &complex.steps;
        !steps.is_empty() && matches_from_right(arena, node_id, steps, steps.len() - 1)
    })
}

fn collect_matches(
    arena: &Arena,
    node_id: u32,
    group: &SelectorGroup,
    results: &mut Vec<u32>,
) {
    let (is_element, first_child) = match arena.get(node_id) {
        Some(node) => (matches!(node.kind, NodeKind::Element { .. }), node.first_child),
        None => return,
    };

    if is_element && element_matches(arena, node_id, group) {
        results.push(node_id);
    }

    let mut child = first_child;
    while let Some(child_id) = child {
        collect_matches(arena, child_id, group, results);
        child = arena.get(child_id).and_then(|n| n.next_sibling);
    }
}

/// Walk the tree in document order from `root_id` and return all elements matching `selector_str`.
pub(crate) fn query_all(arena: &Arena, root_id: u32, selector_str: &str) -> Vec<u32> {
    let group = parse_selector_group(selector_str);
    if group.is_empty() {
        return Vec::new();
    }
    let mut results = Vec::new();
    collect_matches(arena, root_id, &group, &mut results);
    results
}

// ── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_one(s: &str) -> ComplexSelector {
        let mut g = parse_selector_group(s);
        assert_eq!(g.len(), 1, "expected one selector, got {}", g.len());
        g.remove(0)
    }

    #[test]
    fn parses_tag_selector() {
        let s = parse_one("img");
        assert_eq!(s.steps.len(), 1);
        assert_eq!(s.steps[0].simple.tag, Some("img".into()));
    }

    #[test]
    fn parses_class_selector() {
        let s = parse_one(".thumb");
        assert_eq!(s.steps[0].simple.tag, None);
        assert_eq!(s.steps[0].simple.classes, vec!["thumb"]);
    }

    #[test]
    fn parses_id_selector() {
        let s = parse_one("#x");
        assert_eq!(s.steps[0].simple.id, Some("x".into()));
    }

    #[test]
    fn parses_compound_selector() {
        let s = parse_one("img.thumb#hero");
        let step = &s.steps[0];
        assert_eq!(step.simple.tag, Some("img".into()));
        assert_eq!(step.simple.classes, vec!["thumb"]);
        assert_eq!(step.simple.id, Some("hero".into()));
    }

    #[test]
    fn parses_attribute_presence() {
        let s = parse_one("img[src]");
        let attr = &s.steps[0].simple.attrs[0];
        assert_eq!(attr.name, "src");
        assert_eq!(attr.value, None);
    }

    #[test]
    fn parses_attribute_value_quoted() {
        let s = parse_one("img[src=\"/a.png\"]");
        let attr = &s.steps[0].simple.attrs[0];
        assert_eq!(attr.name, "src");
        assert_eq!(attr.value, Some("/a.png".into()));
    }

    #[test]
    fn parses_descendant_combinator() {
        let s = parse_one("main img");
        assert_eq!(s.steps.len(), 2);
        assert_eq!(s.steps[1].combinator, Some(Combinator::Descendant));
    }

    #[test]
    fn parses_child_combinator() {
        let s = parse_one("main > img");
        assert_eq!(s.steps[1].combinator, Some(Combinator::Child));
    }

    #[test]
    fn parses_selector_group() {
        let g = parse_selector_group("nav, footer, .sidebar");
        assert_eq!(g.len(), 3);
    }
}
