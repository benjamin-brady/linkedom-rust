# linkedom-rust: Rust/WASM DOM for Cloudflare Workers

## Problem

linkedom (https://github.com/webreflection/linkedom) doesn't work reliably in Cloudflare Workers due to the restrictive runtime environment. We need a DOM manipulation library that:

- Parses HTML (specifically Wikipedia pages)
- Supports CSS selector queries (querySelector/querySelectorAll)
- Allows DOM mutation (remove elements, modify attributes, create nodes)
- Serializes back to HTML
- Runs in Cloudflare Workers without compatibility issues

## Approach

Build a purpose-built DOM library in Rust, compiled to WASM, using arena-allocated nodes with linked-list indices. This preserves linkedom's O(1) insertion/removal characteristics while being Rust-native. Uses proven Mozilla/Servo crates for parsing and selector matching.

## Architecture

### Core Data Structure: Arena-Based DOM

Nodes are stored in a flat `Vec` (arena). Relationships between nodes are represented as indices, giving O(1) mutations without Rust ownership complexity.

```rust
type NodeId = u32;

struct Document {
    nodes: Vec<Node>,
    // root is always index 0
}

struct Node {
    node_type: NodeType,
    parent: Option<NodeId>,
    first_child: Option<NodeId>,
    last_child: Option<NodeId>,
    prev_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    data: NodeData,
}

enum NodeData {
    Element {
        tag: String,
        attributes: Vec<(String, String)>,
        namespace: Namespace,
    },
    Text(String),
    Comment(String),
    Document,
    DocumentType {
        name: String,
    },
}
```

### Performance Characteristics (matching linkedom)

| Operation         | Complexity | Notes                                      |
| ----------------- | ---------- | ------------------------------------------ |
| appendChild       | O(1)       | Update parent's last_child + sibling links |
| removeChild       | O(1)       | Unlink from sibling chain + parent         |
| insertBefore      | O(1)       | Splice into sibling chain                  |
| querySelector     | O(n)       | Tree walk with selector matching           |
| querySelectorAll  | O(n)       | Full tree walk                             |
| textContent (get) | O(k)       | Walk subtree, k = descendant count         |
| getAttribute      | O(a)       | a = attribute count on element             |
| serialize         | O(n)       | Full tree walk                             |

### Rust Crate Dependencies

- **html5ever** — HTML5 spec-compliant parser (Mozilla/Servo). Handles malformed HTML correctly per spec.
- **selectors** — CSS selector parsing and matching (Servo/Firefox engine). Supports complex selectors.
- **wasm-bindgen** — JS↔WASM interop, generates TypeScript types.
- **wasm-bindgen-futures** (if needed) — async support.

### WASM API Layer (flat functions via wasm-bindgen)

The WASM module exports flat functions. Opaque handles (u32 indices) represent nodes on the JS side.

```rust
#[wasm_bindgen]
pub fn parse_html(html: &str) -> DocumentHandle;

#[wasm_bindgen]
pub fn query_selector(doc: &DocumentHandle, root: u32, selector: &str) -> Option<u32>;

#[wasm_bindgen]
pub fn query_selector_all(doc: &DocumentHandle, root: u32, selector: &str) -> Vec<u32>;

#[wasm_bindgen]
pub fn get_attribute(doc: &DocumentHandle, node: u32, name: &str) -> Option<String>;

#[wasm_bindgen]
pub fn set_attribute(doc: &DocumentHandle, node: u32, name: &str, value: &str);

#[wasm_bindgen]
pub fn remove_attribute(doc: &DocumentHandle, node: u32, name: &str);

#[wasm_bindgen]
pub fn get_text_content(doc: &DocumentHandle, node: u32) -> String;

#[wasm_bindgen]
pub fn set_text_content(doc: &DocumentHandle, node: u32, text: &str);

#[wasm_bindgen]
pub fn get_inner_html(doc: &DocumentHandle, node: u32) -> String;

#[wasm_bindgen]
pub fn set_inner_html(doc: &DocumentHandle, node: u32, html: &str);

#[wasm_bindgen]
pub fn create_element(doc: &mut DocumentHandle, tag: &str) -> u32;

#[wasm_bindgen]
pub fn create_text_node(doc: &mut DocumentHandle, text: &str) -> u32;

#[wasm_bindgen]
pub fn append_child(doc: &mut DocumentHandle, parent: u32, child: u32);

#[wasm_bindgen]
pub fn remove_child(doc: &mut DocumentHandle, parent: u32, child: u32);

#[wasm_bindgen]
pub fn remove(doc: &mut DocumentHandle, node: u32);

#[wasm_bindgen]
pub fn get_tag_name(doc: &DocumentHandle, node: u32) -> String;

#[wasm_bindgen]
pub fn get_children(doc: &DocumentHandle, node: u32) -> Vec<u32>;

#[wasm_bindgen]
pub fn class_list_add(doc: &mut DocumentHandle, node: u32, cls: &str);

#[wasm_bindgen]
pub fn class_list_remove(doc: &mut DocumentHandle, node: u32, cls: &str);

#[wasm_bindgen]
pub fn class_list_contains(doc: &DocumentHandle, node: u32, cls: &str) -> bool;

#[wasm_bindgen]
pub fn serialize(doc: &DocumentHandle) -> String;
```

### TypeScript Wrapper (runs in Workers, calls WASM)

A thin OOP wrapper provides a familiar DOM-like API:

```typescript
import init, * as wasm from "./linkedom_rust_bg.wasm";

export class Document {
  private handle: wasm.DocumentHandle;

  static parse(html: string): Document {
    return new Document(wasm.parse_html(html));
  }

  querySelector(selector: string): Element | null {
    const id = wasm.query_selector(this.handle, 0, selector);
    return id != null ? new Element(this.handle, id) : null;
  }

  querySelectorAll(selector: string): Element[] {
    const ids = wasm.query_selector_all(this.handle, 0, selector);
    return ids.map((id) => new Element(this.handle, id));
  }

  createElement(tag: string): Element {
    const id = wasm.create_element(this.handle, tag);
    return new Element(this.handle, id);
  }

  serialize(): string {
    return wasm.serialize(this.handle);
  }
}

export class Element {
  constructor(
    private doc: wasm.DocumentHandle,
    private id: number,
  ) {}

  querySelector(selector: string): Element | null {
    const id = wasm.query_selector(this.doc, this.id, selector);
    return id != null ? new Element(this.doc, id) : null;
  }

  querySelectorAll(selector: string): Element[] {
    return wasm
      .query_selector_all(this.doc, this.id, selector)
      .map((id) => new Element(this.doc, id));
  }

  getAttribute(name: string): string | null {
    return wasm.get_attribute(this.doc, this.id, name);
  }

  setAttribute(name: string, value: string): void {
    wasm.set_attribute(this.doc, this.id, name, value);
  }

  get textContent(): string {
    return wasm.get_text_content(this.doc, this.id);
  }

  set textContent(text: string) {
    wasm.set_text_content(this.doc, this.id, text);
  }

  get innerHTML(): string {
    return wasm.get_inner_html(this.doc, this.id);
  }

  set innerHTML(html: string) {
    wasm.set_inner_html(this.doc, this.id, html);
  }

  get tagName(): string {
    return wasm.get_tag_name(this.doc, this.id);
  }

  get children(): Element[] {
    return wasm
      .get_children(this.doc, this.id)
      .map((id) => new Element(this.doc, id));
  }

  get classList(): ClassList {
    return new ClassList(this.doc, this.id);
  }

  appendChild(child: Element): void {
    wasm.append_child(this.doc, this.id, child.id);
  }

  remove(): void {
    wasm.remove(this.doc, this.id);
  }
}

class ClassList {
  constructor(
    private doc: wasm.DocumentHandle,
    private id: number,
  ) {}
  add(cls: string) {
    wasm.class_list_add(this.doc, this.id, cls);
  }
  remove(cls: string) {
    wasm.class_list_remove(this.doc, this.id, cls);
  }
  contains(cls: string): boolean {
    return wasm.class_list_contains(this.doc, this.id, cls);
  }
}
```

## Cloudflare Workers Integration

### Build Pipeline

1. `cargo build --target wasm32-unknown-unknown --release`
2. `wasm-bindgen --target web` generates JS glue + `.d.ts`
3. `wasm-opt -Oz` for size optimization
4. Bundle with wrangler as ES module

### wrangler.toml

```toml
name = "my-worker"
main = "src/index.ts"
compatibility_date = "2024-01-01"

[build]
command = "npm run build"

[[rules]]
type = "CompiledWasm"
globs = ["**/*.wasm"]
```

### Worker Usage Example

```typescript
import { Document } from "./linkedom-rust";

export default {
  async fetch(request: Request): Promise<Response> {
    const resp = await fetch(
      "https://en.wikipedia.org/wiki/Rust_(programming_language)",
    );
    const html = await resp.text();

    const doc = Document.parse(html);

    // Extract all images
    const images = doc.querySelectorAll("img");
    const srcs = images.map((img) => img.getAttribute("src"));

    // Strip out navigation, footers, etc.
    doc.querySelectorAll("nav, footer, .sidebar").forEach((el) => el.remove());

    // Get clean content
    const clean = doc.serialize();

    return new Response(JSON.stringify({ images: srcs, html: clean }), {
      headers: { "Content-Type": "application/json" },
    });
  },
};
```

## Size Budget

- **html5ever** tokenizer + tree builder: ~300-600KB WASM
- **selectors** crate: ~100-200KB WASM
- **Our DOM code**: ~50-100KB WASM
- **Total estimate**: 500KB-1MB after `wasm-opt -Oz`
- **Workers limit**: 10MB (paid), 1MB (free) — should fit paid plan comfortably; free tier is tight

### Size Mitigation Strategies (if needed)

- Use `wasm-opt -Oz` aggressively
- Enable LTO in Cargo.toml (`lto = true`)
- Strip debug info (`strip = "symbols"`)
- Consider `wee_alloc` for smaller allocator
- If still too large: swap html5ever for a simpler parser (e.g., `tl` crate) at the cost of spec compliance

## Risks and Open Questions

1. **html5ever + selectors integration**: The `selectors` crate expects a trait implementation (`TElement`, `TNode`) on your tree. We need to implement these traits for our arena-based tree. This is well-documented but non-trivial.

2. **Memory management**: WASM linear memory grows but doesn't shrink. For large Wikipedia pages, parsing could allocate significant memory. Document handles should be explicitly freed.

3. **String passing overhead**: Every string crosses the WASM boundary (copy). For heavy attribute reading, this could add up. Mitigation: batch operations where possible.

4. **Free tier size**: May not fit in 1MB. Need to measure early and decide if free-tier support matters.

5. **innerHTML parsing**: Setting innerHTML requires re-invoking the parser on a fragment. html5ever supports this but needs careful context handling.

## Project Structure

```
linkedom-rust/
├── Cargo.toml
├── src/
│   ├── lib.rs          # wasm_bindgen exports
│   ├── arena.rs        # Node arena (Vec<Node>)
│   ├── node.rs         # Node types and data
│   ├── tree.rs         # Tree operations (append, remove, etc.)
│   ├── parser.rs       # html5ever integration
│   ├── selector.rs     # selectors crate trait impls
│   ├── serialize.rs    # HTML serialization
│   └── class_list.rs   # classList operations
├── js/
│   ├── index.ts        # TypeScript wrapper classes
│   └── package.json    # npm package config
├── tests/
│   ├── parse_test.rs
│   ├── selector_test.rs
│   └── mutation_test.rs
└── worker-example/
    ├── src/index.ts
    └── wrangler.toml
```

## Success Criteria

1. Parse a full Wikipedia page HTML without errors
2. querySelectorAll("img") returns correct results
3. Element removal + serialize produces valid HTML
4. Works in Cloudflare Workers without runtime errors
5. WASM bundle < 2MB (ideally < 1MB)
6. Parse + query + serialize a Wikipedia page in < 100ms
