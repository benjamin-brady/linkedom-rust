# linkedom-rust WASM DOM Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust/WASM DOM manipulation library for Cloudflare Workers that parses HTML, queries elements, mutates the tree, and serializes HTML.

**Architecture:** Use an arena-backed DOM where each node stores parent/child/sibling indices to preserve linkedom-style O(1) mutation. Parse HTML with `html5ever`, implement a practical CSS selector engine for the initial Workers/Wikipedia scope, expose a flat `wasm-bindgen` API, and layer a TypeScript wrapper over it.

**Tech Stack:** Rust, wasm-bindgen, html5ever, markup5ever, TypeScript wrapper, wasm-pack-compatible Cargo configuration, cargo tests.

---

## File Structure

- `Cargo.toml`: crate metadata, wasm/staticlib targets, dependencies, release size settings.
- `src/node.rs`: `NodeId`, `Node`, `NodeData`, `ElementData`, and attribute helpers.
- `src/arena.rs`: arena allocation, node lookup, creation helpers, and validity checks.
- `src/tree.rs`: O(1) tree mutation and traversal helpers.
- `src/parser.rs`: `html5ever` tree sink integration and fragment parsing.
- `src/selector.rs`: focused selector parser/matcher supporting tag, id, class, attribute presence/value, descendant combinators, comma groups, and simple child combinators.
- `src/serialize.rs`: HTML serialization and escaping.
- `src/class_list.rs`: `class` attribute token manipulation.
- `src/lib.rs`: public document type plus `wasm-bindgen` exports.
- `js/package.json`: TypeScript wrapper package metadata.
- `js/index.ts`: ergonomic DOM-like wrapper for Workers.
- `worker-example/wrangler.toml`: Workers example configuration.
- `worker-example/src/index.ts`: Wikipedia parsing/manipulation example.
- `tests/dom_test.rs`: parse/query/mutation/serialization tests.
- `README.md`: usage, build, and Workers notes.

## Task 1: Project Skeleton and Core Types

**Files:**
- Create: `Cargo.toml`
- Create: `src/node.rs`
- Create: `src/arena.rs`
- Create: `src/lib.rs`
- Test: `tests/dom_test.rs`

- [ ] **Step 1: Write failing tests**

```rust
use linkedom_rust::Document;

#[test]
fn creates_document_root() {
    let doc = Document::new();
    assert_eq!(doc.root_id(), 0);
    assert_eq!(doc.node_count(), 1);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test creates_document_root`
Expected: FAIL because the crate and `Document` do not exist yet.

- [ ] **Step 3: Add core crate files**

Create `Cargo.toml` with lib crate settings and initial dependencies. Create `src/node.rs` for node data, `src/arena.rs` for arena storage, and `src/lib.rs` exposing `Document::new`, `root_id`, and `node_count`.

- [ ] **Step 4: Run test**

Run: `cargo test creates_document_root`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src tests
git commit -m "feat: add arena DOM skeleton"
```

## Task 2: O(1) Tree Mutation

**Files:**
- Modify: `src/lib.rs`
- Create: `src/tree.rs`
- Test: `tests/dom_test.rs`

- [ ] **Step 1: Write failing tests**

```rust
#[test]
fn append_and_remove_are_linked_correctly() {
    let mut doc = Document::new();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    doc.append_child(0, div).unwrap();
    doc.append_child(0, span).unwrap();
    assert_eq!(doc.children(0), vec![div, span]);
    doc.remove(span).unwrap();
    assert_eq!(doc.children(0), vec![div]);
    assert_eq!(doc.parent(span), None);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test append_and_remove_are_linked_correctly`
Expected: FAIL because mutation APIs do not exist.

- [ ] **Step 3: Implement mutation helpers**

Add `append_child`, `remove`, `children`, and `parent` so sibling/parent pointers update in constant time.

- [ ] **Step 4: Run test**

Run: `cargo test append_and_remove_are_linked_correctly`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src tests
git commit -m "feat: add arena tree mutation"
```

## Task 3: HTML Parsing and Serialization

**Files:**
- Modify: `src/lib.rs`
- Create: `src/parser.rs`
- Create: `src/serialize.rs`
- Test: `tests/dom_test.rs`

- [ ] **Step 1: Write failing tests**

```rust
#[test]
fn parses_and_serializes_html() {
    let doc = Document::parse("<main><h1>Hello</h1><img src=\"/x.png\"></main>").unwrap();
    assert!(doc.serialize().contains("<main>"));
    assert!(doc.serialize().contains("<h1>Hello</h1>"));
    assert!(doc.serialize().contains("<img src=\"/x.png\">"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test parses_and_serializes_html`
Expected: FAIL because parser/serializer do not exist.

- [ ] **Step 3: Implement html5ever parser and serializer**

Implement `TreeSink` for a document builder and add recursive HTML serialization with text/attribute escaping.

- [ ] **Step 4: Run test**

Run: `cargo test parses_and_serializes_html`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src tests
git commit -m "feat: parse and serialize html"
```

## Task 4: Selectors and DOM APIs

**Files:**
- Modify: `src/lib.rs`
- Create: `src/selector.rs`
- Create: `src/class_list.rs`
- Test: `tests/dom_test.rs`

- [ ] **Step 1: Write failing tests**

```rust
#[test]
fn queries_and_mutates_wikipedia_like_html() {
    let mut doc = Document::parse("<main><img class=\"thumb\" src=\"/a.png\"><p id=\"x\">Text</p><footer>Bye</footer></main>").unwrap();
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test queries_and_mutates_wikipedia_like_html`
Expected: FAIL because selectors and classList APIs do not exist.

- [ ] **Step 3: Implement practical selector matcher and DOM methods**

Support tag, `.class`, `#id`, `[attr]`, `[attr=value]`, grouped selectors with commas, descendant combinators, and direct-child combinators. Add attribute, textContent, innerHTML, classList, and query APIs.

- [ ] **Step 4: Run test**

Run: `cargo test queries_and_mutates_wikipedia_like_html`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src tests
git commit -m "feat: add selectors and DOM mutation APIs"
```

## Task 5: WASM Exports and TypeScript Wrapper

**Files:**
- Modify: `src/lib.rs`
- Create: `js/package.json`
- Create: `js/index.ts`
- Test: `cargo test`

- [ ] **Step 1: Write wasm-facing API**

Expose `WasmDocument` with `parse_html`, `query_selector`, `query_selector_all`, attributes, textContent, innerHTML, createElement, createTextNode, appendChild, remove, children, classList, and serialize.

- [ ] **Step 2: Write TypeScript wrapper**

Create `Document`, `Element`, and `ClassList` classes that call the flat WASM APIs and hide numeric handles from consumers.

- [ ] **Step 3: Run Rust tests**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml src js
git commit -m "feat: expose wasm API and TypeScript wrapper"
```

## Task 6: Worker Example, Docs, GitHub Repo

**Files:**
- Create: `README.md`
- Create: `worker-example/wrangler.toml`
- Create: `worker-example/src/index.ts`

- [ ] **Step 1: Add README and Worker example**

Document Rust tests, WASM build commands, Workers notes, and a Wikipedia image-stripping example.

- [ ] **Step 2: Run final verification**

Run: `cargo test`
Expected: all tests pass.

Run if target is installed: `cargo build --target wasm32-unknown-unknown --release`
Expected: release WASM build succeeds.

- [ ] **Step 3: Create private GitHub repo and push**

Run:
```bash
gh repo create linkedom-rust --private --source=. --remote=origin --push
```

Expected: GitHub repository is created under the authenticated account and local commits are pushed.

- [ ] **Step 4: Commit docs/example before push if needed**

```bash
git add README.md worker-example
git commit -m "docs: add worker usage example"
```

## Self-Review

- Spec coverage: the plan covers arena DOM, linkedom-style O(1) mutation, HTML parsing, selector queries, attribute/text/innerHTML/classList APIs, TypeScript wrapper, Workers example, tests, and GitHub repo creation.
- Placeholder scan: no task relies on TBD/TODO placeholders; each task has exact files, test intent, commands, and expected results.
- Type consistency: public Rust API centers on `Document`; WASM wrapper centers on `WasmDocument`; TypeScript wrappers hide numeric node ids.
