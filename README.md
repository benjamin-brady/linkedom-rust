# linkedom-rust

A Rust + WebAssembly DOM library for environments where running JavaScript DOM libraries is awkward — primarily Cloudflare Workers.

> **Status:** Early work-in-progress. Design and plan are written; implementation is in progress. Not yet usable. Expect breaking changes and missing pieces.

## Why

[linkedom](https://github.com/WebReflection/linkedom) is a great pure-JS DOM implementation, but it doesn't run reliably inside the Cloudflare Workers runtime. I parse Wikipedia HTML inside Workers for [Redactle](https://redactle.net) and wanted a DOM library that:

- Parses real-world (often malformed) HTML correctly
- Supports CSS selectors (`querySelector` / `querySelectorAll`)
- Allows mutation: remove, modify attributes, create nodes, etc.
- Serializes back to HTML
- Works inside the Workers runtime without compatibility headaches

A Rust crate compiled to WASM is a natural fit — and a good excuse to learn Rust on a non-trivial codebase.

## Approach

- **Arena-backed DOM.** Nodes live in a flat `Vec<Node>` and reference each other by `u32` indices (parent, first/last child, prev/next sibling). This preserves linkedom's O(1) mutation performance while sidestepping Rust ownership pain on a graph-shaped data structure.
- **Parsing:** [`html5ever`](https://crates.io/crates/html5ever) (Mozilla/Servo) for spec-compliant HTML5 parsing.
- **Selectors:** a focused selector engine targeting the cases I actually need (tag, id, class, attribute, descendant and child combinators, comma groups). Larger selector support can come later via [`selectors`](https://crates.io/crates/selectors) if needed.
- **WASM boundary:** a flat `wasm-bindgen` API surface using opaque `u32` node handles, with a small TypeScript wrapper that gives the JS side an ergonomic DOM-like API.

### Performance targets

| Operation         | Complexity |
| ----------------- | ---------- |
| `appendChild`     | O(1)       |
| `removeChild`     | O(1)       |
| `insertBefore`    | O(1)       |
| `querySelector`   | O(n)       |
| `querySelectorAll`| O(n)       |
| `getAttribute`    | O(a)       |
| `serialize`       | O(n)       |

## Project layout

```
src/
  arena.rs       arena storage and node lookup
  node.rs        Node, NodeData, ElementData
  tree.rs        O(1) tree mutation / traversal
  parser.rs      html5ever tree sink integration
  selector.rs    selector parser and matcher
  serialize.rs   HTML serialization + escaping
  class_list.rs  class attribute manipulation
  lib.rs         public Document API + wasm-bindgen exports
js/              TypeScript wrapper package
worker-example/  Cloudflare Workers example
tests/           cargo tests
docs/            design + implementation plan
```

## Building

Requires Rust (stable) and [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/).

```bash
# Run native tests
cargo test

# Build the WASM package
wasm-pack build --target web
```

## Status checklist

- [ ] Arena and core node types
- [ ] HTML parsing (html5ever integration)
- [ ] Tree mutation API
- [ ] CSS selector engine (subset)
- [ ] HTML serialization
- [ ] `wasm-bindgen` API surface
- [ ] TypeScript wrapper
- [ ] Cloudflare Workers example
- [ ] Benchmarks vs linkedom

## Notes

- This is a personal learning project. The design and architecture are mine; a fair amount of the Rust code is AI-assisted, with my role focused on architecture, review, and learning idiomatic Rust patterns.
- See [docs/superpowers/specs/](docs/superpowers/specs/) for the full design and [docs/superpowers/plans/](docs/superpowers/plans/) for the implementation plan.

## License

MIT (planned).
