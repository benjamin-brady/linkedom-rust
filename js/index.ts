/**
 * linkedom-rust TypeScript wrapper
 *
 * Wraps the wasm-pack-generated WASM bindings (`../pkg/`) with ergonomic
 * `Document`, `Element`, and `ClassList` classes that hide numeric node ids.
 *
 * ## Setup
 *
 * 1. Build the WASM package:
 *    ```
 *    wasm-pack build .. --target web --out-dir ../pkg
 *    ```
 * 2. Call `init()` once before using any DOM classes.
 *
 * ## Worker / ESM usage
 *
 * ```ts
 * import { init, Document } from './index.js';
 * await init();
 * const doc = Document.parse('<h1>Hello</h1>');
 * console.log(doc.serialize());
 * ```
 *
 * @module
 */

// ---------------------------------------------------------------------------
// Raw WASM binding types
// ---------------------------------------------------------------------------

/**
 * Shape of the `WasmDocument` class emitted by wasm-pack.
 *
 * wasm-bindgen generates camelCase JS names from the Rust `#[wasm_bindgen(js_name = …)]`
 * annotations in `src/wasm.rs`.  Node ids are plain JS numbers.
 *
 * ## Fallible methods
 *
 * Rust methods that return `Result<T, JsValue>` are exposed here with return type `T`
 * (e.g., `void` for `Result<(), …>`, `boolean` for `Result<bool, …>`).
 * wasm-bindgen converts the `Err` variant into a thrown JavaScript `Error`, so callers
 * receive the success value directly and errors propagate as exceptions — matching the
 * browser DOM convention.  There is no `| never` union; exceptions are the JS idiom.
 */
interface IRawDocument {
  // serialisation
  serialize(): string;
  // node factory
  createElement(tag: string): number;
  createTextNode(text: string): number;
  // tree
  appendChild(parent: number, child: number): void;
  remove(node: number): void;
  children(node: number): Uint32Array;
  // queries
  querySelector(selector: string): number | undefined;
  querySelectorAll(selector: string): Uint32Array;
  // attributes
  getAttribute(node: number, name: string): string | undefined;
  setAttribute(node: number, name: string, value: string): void;
  removeAttribute(node: number, name: string): void;
  // text content
  getTextContent(node: number): string | undefined;
  setTextContent(node: number, text: string): void;
  // innerHTML
  getInnerHtml(node: number): string | undefined;
  setInnerHtml(node: number, html: string): void;
  // classList
  classListAdd(node: number, cls: string): void;
  classListRemove(node: number, cls: string): void;
  classListContains(node: number, cls: string): boolean;
  // misc
  rootId(): number;
}

/** Static side of the wasm-pack-generated `WasmDocument` class. */
interface IRawDocumentStatic {
  parseHtml(html: string): IRawDocument;
}

/** Shape of the module object returned by wasm-pack (`../pkg/linkedom_rust.js`). */
interface IWasmModule {
  /**
   * The wasm-pack default export that initialises the WASM binary.
   *
   * The full wasm-bindgen signature also accepts `BufferSource` and
   * `WebAssembly.Module`, but those require pre-fetched or pre-compiled bytes.
   * We intentionally narrow to `string | URL` here because those are the only
   * meaningful inputs when loading from a URL in a browser/Worker context.
   * Callers that need `BufferSource` / `WebAssembly.Module` should call the
   * raw module directly instead of going through this wrapper.
   */
  default(input?: string | URL): Promise<void>;
  WasmDocument: IRawDocumentStatic;
}

// ---------------------------------------------------------------------------
// Module initialisation
// ---------------------------------------------------------------------------

let _mod: IWasmModule | null = null;
/** Cached promise so that concurrent `init()` calls share a single load. */
let _initPromise: Promise<void> | null = null;

/**
 * Initialise the WASM module.  Must be called (and awaited) once before
 * creating any `Document` instances.
 *
 * Concurrent callers receive the same promise so the binary is never
 * fetched or compiled twice.
 *
 * @param wasmPath  Optional explicit URL or path to the `.wasm` binary.
 *                  When omitted the URL embedded by wasm-pack is used.
 */
export function init(wasmPath?: string | URL): Promise<void> {
  if (_mod !== null) return Promise.resolve();
  if (_initPromise !== null) return _initPromise;

  // Dynamic import keeps the module Worker / ESM-friendly: bundlers can
  // tree-shake the wasm initialisation and Workers can control when the
  // module is loaded.
  _initPromise = (async () => {
    const m = (await import('../pkg/linkedom_rust.js')) as IWasmModule;
    await m.default(wasmPath);
    _mod = m;
  })();
  return _initPromise;
}

function requireMod(): IWasmModule {
  if (_mod === null) {
    throw new Error('linkedom-rust: call `await init()` before using DOM classes');
  }
  return _mod;
}

// ---------------------------------------------------------------------------
// ClassList
// ---------------------------------------------------------------------------

/**
 * Mirrors a subset of the browser `DOMTokenList` API for a single element.
 */
export class ClassList {
  readonly #raw: IRawDocument;
  readonly #nodeId: number;

  /** @internal */
  constructor(raw: IRawDocument, nodeId: number) {
    this.#raw = raw;
    this.#nodeId = nodeId;
  }

  /** Add a CSS class token.  No-op if already present. */
  add(token: string): void {
    this.#raw.classListAdd(this.#nodeId, token);
  }

  /** Remove a CSS class token.  No-op if absent. */
  remove(token: string): void {
    this.#raw.classListRemove(this.#nodeId, token);
  }

  /** Return `true` if the element has the given class token. */
  contains(token: string): boolean {
    return this.#raw.classListContains(this.#nodeId, token);
  }
}

// ---------------------------------------------------------------------------
// Element
// ---------------------------------------------------------------------------

/**
 * Wraps a numeric WASM node id and a reference to the owning `Document`,
 * providing a DOM-like element API without exposing raw ids to consumers.
 */
export class Element {
  readonly #raw: IRawDocument;
  readonly #nodeId: number;

  /**
   * @internal – read the raw node id; only for same-module use by `Document`.
   * This getter is read-only; the backing field cannot be mutated from outside.
   */
  get _nodeId(): number {
    return this.#nodeId;
  }

  /** @internal */
  constructor(raw: IRawDocument, nodeId: number) {
    this.#raw = raw;
    this.#nodeId = nodeId;
  }

  // ── Attributes ─────────────────────────────────────────────────────────

  getAttribute(name: string): string | undefined {
    return this.#raw.getAttribute(this.#nodeId, name);
  }

  setAttribute(name: string, value: string): void {
    this.#raw.setAttribute(this.#nodeId, name, value);
  }

  removeAttribute(name: string): void {
    this.#raw.removeAttribute(this.#nodeId, name);
  }

  // ── Text / HTML content ─────────────────────────────────────────────────

  get textContent(): string | undefined {
    return this.#raw.getTextContent(this.#nodeId);
  }

  set textContent(value: string) {
    this.#raw.setTextContent(this.#nodeId, value);
  }

  get innerHTML(): string | undefined {
    return this.#raw.getInnerHtml(this.#nodeId);
  }

  set innerHTML(value: string) {
    this.#raw.setInnerHtml(this.#nodeId, value);
  }

  // ── classList ───────────────────────────────────────────────────────────

  get classList(): ClassList {
    return new ClassList(this.#raw, this.#nodeId);
  }

  // ── Tree ────────────────────────────────────────────────────────────────

  /**
   * Return all direct child nodes as `Element` wrappers.
   *
   * **Note:** despite the name, this returns *all* child node types (text nodes,
   * comment nodes, element nodes, …) — analogous to the browser's `childNodes`
   * property rather than the element-only `children`.  An additive alias
   * `childNodes` is provided below; prefer it in new code.
   */
  get children(): Element[] {
    const ids = this.#raw.children(this.#nodeId);
    return Array.from(ids).map((id) => new Element(this.#raw, id));
  }

  /**
   * Alias for {@link children}.  Returns all direct child node wrappers
   * (elements, text nodes, etc.) — equivalent to the browser's `childNodes`.
   */
  get childNodes(): Element[] {
    return this.children;
  }

  /** Append `child` as the last child of this element. */
  append(child: Element): void {
    this.#raw.appendChild(this.#nodeId, child.#nodeId);
  }

  /** Detach this element from its parent. */
  remove(): void {
    this.#raw.remove(this.#nodeId);
  }
}

// ---------------------------------------------------------------------------
// Document
// ---------------------------------------------------------------------------

/**
 * The top-level DOM document.  Wraps a `WasmDocument` instance and exposes
 * ergonomic factory methods and query APIs.
 *
 * @example
 * ```ts
 * import { init, Document } from './index.js';
 * await init();
 *
 * const doc = Document.parse('<main><p id="intro">Hello</p></main>');
 * const p = doc.querySelector('#intro');
 * p?.setAttribute('class', 'highlight');
 * console.log(doc.serialize());
 * ```
 */
export class Document {
  readonly #raw: IRawDocument;

  /** @internal */
  private constructor(raw: IRawDocument) {
    this.#raw = raw;
  }

  // ── Construction ─────────────────────────────────────────────────────────

  /** Parse an HTML string and return a new `Document`. */
  static parse(html: string): Document {
    return new Document(requireMod().WasmDocument.parseHtml(html));
  }

  // ── Serialization ─────────────────────────────────────────────────────────

  /** Serialize the document to an HTML string. */
  serialize(): string {
    return this.#raw.serialize();
  }

  // ── Node factory ─────────────────────────────────────────────────────────

  /** Create a detached element with the given tag name. */
  createElement(tag: string): Element {
    return new Element(this.#raw, this.#raw.createElement(tag));
  }

  /**
   * Create a detached text node with the given content.
   *
   * **Return-type hazard:** the returned `Element` wrapper is a text node, not
   * an element node.  Calling attribute methods (`getAttribute`, `setAttribute`,
   * `classList`, …) on it will throw at the WASM boundary.  Only `textContent`,
   * `append`, and `remove` are safe on a text-node wrapper.
   */
  createTextNode(text: string): Element {
    return new Element(this.#raw, this.#raw.createTextNode(text));
  }

  // ── Queries ───────────────────────────────────────────────────────────────

  /** Return the first element matching `selector`, or `undefined`. */
  querySelector(selector: string): Element | undefined {
    const id = this.#raw.querySelector(selector);
    return id !== undefined ? new Element(this.#raw, id) : undefined;
  }

  /** Return all elements matching `selector` in document order. */
  querySelectorAll(selector: string): Element[] {
    const ids = this.#raw.querySelectorAll(selector);
    return Array.from(ids).map((id) => new Element(this.#raw, id));
  }

  // ── Tree ─────────────────────────────────────────────────────────────────

  /** Append `child` as the last child of `parent`. */
  appendChild(parent: Element, child: Element): void {
    this.#raw.appendChild(parent._nodeId, child._nodeId);
  }

  /** Detach `node` from its parent. */
  removeChild(node: Element): void {
    this.#raw.remove(node._nodeId);
  }
}
