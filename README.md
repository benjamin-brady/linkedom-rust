# linkedom-rust

A Rust/WebAssembly DOM implementation designed for [Cloudflare Workers], inspired by
the data-structure trade-offs in [linkedom] (flat arena storage, numeric node ids,
O(1) node removal).

## Purpose

Parse, query, and manipulate HTML server-side using the same mental model as the
browser DOM — without shipping a full JS HTML parser to every edge Worker.  The
project provides:

| Layer | Path | Description |
|---|---|---|
| Rust DOM | `src/` | Arena-based tree, CSS selector engine, html5ever parser, serialiser |
| WASM shim | `src/wasm.rs` | `wasm-bindgen` exports; only compiled for `wasm32` targets |
| TS wrapper | `js/index.ts` | Ergonomic `Document`/`Element`/`ClassList` hiding raw node ids |
| Worker example | `worker-example/` | Cloudflare Worker that strips boilerplate from a Wikipedia page |

## Status and Limitations

- **Native Rust layer** — fully implemented and tested.  All 49 tests pass.
- **WASM shim** — written and type-checked; correct by inspection.
- **Local WASM build** — blocked: `wasm32-unknown-unknown` stdlib requires
  `rustup` (or an equivalent toolchain manager).  The project Homebrew `rustc`
  does not include it.  See [WASM build](#wasm-build) below.
- **html5ever / string_cache / parking_lot WASM compatibility** — not yet
  validated.  These crates are widely used in WASM contexts but must be
  confirmed once the target is installed.
- **Cloudflare Workers runtime** — not yet proven end-to-end; `worker-example/`
  is provided as a reference implementation pending a successful WASM build.

## Verification

### Rust

```sh
cargo test
cargo clippy -- -D warnings
```

### TypeScript

```sh
cd js && npm run typecheck
```

Both commands pass on the current HEAD without a WASM build.

## WASM Build

**Intended path** (requires rustup + wasm-pack):

```sh
# 1. Install the WASM target (one-time)
rustup target add wasm32-unknown-unknown

# 2. Build the WASM package
cd js && npm run build:wasm
# Equivalent: wasm-pack build .. --target web --out-dir ../pkg
```

**Current blocker** — The machine running this project uses a Homebrew-installed
`rustc` which does not support `rustup target add`.  Until the WASM target stdlib
is available, `cargo build --target wasm32-unknown-unknown` will fail with a
`can't find crate for std` error.  Installing `rustup` (https://rustup.rs) and
re-running the commands above will resolve this.

## Rust Usage

```rust
use linkedom_rust::Document;

// Parse
let mut doc = Document::parse(r#"
  <html>
    <body>
      <nav id="nav">…</nav>
      <article>
        <img src="photo.jpg" alt="A photo">
        <img src="logo.png" alt="Logo">
      </article>
      <footer id="footer">…</footer>
    </body>
  </html>
"#);

// Collect image srcs
let img_ids = doc.query_selector_all("img");
let srcs: Vec<String> = img_ids
    .iter()
    .filter_map(|&id| doc.get_attribute(id, "src"))
    .collect();

// Remove navigation chrome
for selector in &["nav", "footer", "aside", "#sidebar"] {
    if let Some(id) = doc.query_selector(selector) {
        doc.remove(id).ok();
    }
}

// Serialise cleaned document
let html = doc.serialize();
```

## TypeScript / Worker Usage

```ts
import { init, Document } from '@linkedom-rust/dom';

// ⚠ Requires a successful `npm run build:wasm` first.
await init();

const doc = Document.parse('<html><body><img src="a.jpg"><nav>…</nav></body></html>');

// Collect image srcs
const imgs = doc.querySelectorAll('img');
const srcs = imgs.map(el => el.getAttribute('src')).filter(Boolean);

// Strip navigation chrome
for (const sel of ['nav', 'footer', 'aside', '#sidebar']) {
  doc.querySelectorAll(sel).forEach(el => el.remove());
}

// Serialise
console.log(doc.serialize());
```

See [`worker-example/src/index.ts`](worker-example/src/index.ts) for a full
Cloudflare Worker implementation.

## Project Structure

```
src/
  lib.rs          # Public Rust API + Document
  arena.rs        # Flat node arena (Vec<Node>)
  node.rs         # NodeKind enum (Document, Element, Text, …)
  tree.rs         # append_child, remove_node, traversal helpers
  parser.rs       # html5ever-based HTML parser
  selector.rs     # CSS selector engine (element, class, id, attribute, combinator)
  serialize.rs    # HTML serialiser
  class_list.rs   # classList helpers
  wasm.rs         # wasm-bindgen shim (wasm32 only)
js/
  index.ts        # TypeScript wrapper (Document, Element, ClassList)
  package.json
  tsconfig.json
pkg/
  linkedom_rust.d.ts  # Type stub (placeholder until wasm-pack generates the real one)
worker-example/
  wrangler.toml   # Cloudflare Worker config
  src/index.ts    # Worker implementation
tests/
  dom_test.rs     # Integration tests
```

[linkedom]: https://github.com/WebReflection/linkedom
[Cloudflare Workers]: https://workers.cloudflare.com/
