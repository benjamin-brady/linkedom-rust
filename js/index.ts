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
  default(input?: string | URL | BufferSource | WebAssembly.Module): Promise<void>;
  WasmDocument: IRawDocumentStatic;
}

// ---------------------------------------------------------------------------
// Module initialisation
// ---------------------------------------------------------------------------

let _mod: IWasmModule | null = null;

/**
 * Initialise the WASM module.  Must be called (and awaited) once before
 * creating any `Document` instances.
 *
 * @param wasmPath  Optional explicit URL or path to the `.wasm` binary.
 *                  When omitted the URL embedded by wasm-pack is used.
 */
export async function init(wasmPath?: string | URL): Promise<void> {
  if (_mod !== null) return; // already initialised

  // Dynamic import keeps the module Worker / ESM-friendly: bundlers can
  // tree-shake the wasm initialisation and Workers can control when the
  // module is loaded.
  const m = (await import('../pkg/linkedom_rust.js')) as IWasmModule;
  await m.default(wasmPath);
  _mod = m;
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
  /** @internal */
  readonly _nodeId: number;
  /** @internal */
  readonly _raw: IRawDocument;

  /** @internal */
  constructor(raw: IRawDocument, nodeId: number) {
    this._raw = raw;
    this._nodeId = nodeId;
  }

  // ── Attributes ─────────────────────────────────────────────────────────

  getAttribute(name: string): string | undefined {
    return this._raw.getAttribute(this._nodeId, name);
  }

  setAttribute(name: string, value: string): void {
    this._raw.setAttribute(this._nodeId, name, value);
  }

  removeAttribute(name: string): void {
    this._raw.removeAttribute(this._nodeId, name);
  }

  // ── Text / HTML content ─────────────────────────────────────────────────

  get textContent(): string | undefined {
    return this._raw.getTextContent(this._nodeId);
  }

  set textContent(value: string) {
    this._raw.setTextContent(this._nodeId, value);
  }

  get innerHTML(): string | undefined {
    return this._raw.getInnerHtml(this._nodeId);
  }

  set innerHTML(value: string) {
    this._raw.setInnerHtml(this._nodeId, value);
  }

  // ── classList ───────────────────────────────────────────────────────────

  get classList(): ClassList {
    return new ClassList(this._raw, this._nodeId);
  }

  // ── Tree ────────────────────────────────────────────────────────────────

  /** Return direct children as `Element` instances. */
  get children(): Element[] {
    const ids = this._raw.children(this._nodeId);
    return Array.from(ids).map((id) => new Element(this._raw, id));
  }

  /** Append `child` as the last child of this element. */
  append(child: Element): void {
    this._raw.appendChild(this._nodeId, child._nodeId);
  }

  /** Detach this element from its parent. */
  remove(): void {
    this._raw.remove(this._nodeId);
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
  /** @internal */
  readonly _raw: IRawDocument;

  /** @internal */
  private constructor(raw: IRawDocument) {
    this._raw = raw;
  }

  // ── Construction ─────────────────────────────────────────────────────────

  /** Parse an HTML string and return a new `Document`. */
  static parse(html: string): Document {
    return new Document(requireMod().WasmDocument.parseHtml(html));
  }

  // ── Serialization ─────────────────────────────────────────────────────────

  /** Serialize the document to an HTML string. */
  serialize(): string {
    return this._raw.serialize();
  }

  // ── Node factory ─────────────────────────────────────────────────────────

  /** Create a detached element with the given tag name. */
  createElement(tag: string): Element {
    return new Element(this._raw, this._raw.createElement(tag));
  }

  /** Create a detached text node with the given content. */
  createTextNode(text: string): Element {
    return new Element(this._raw, this._raw.createTextNode(text));
  }

  // ── Queries ───────────────────────────────────────────────────────────────

  /** Return the first element matching `selector`, or `undefined`. */
  querySelector(selector: string): Element | undefined {
    const id = this._raw.querySelector(selector);
    return id !== undefined ? new Element(this._raw, id) : undefined;
  }

  /** Return all elements matching `selector` in document order. */
  querySelectorAll(selector: string): Element[] {
    const ids = this._raw.querySelectorAll(selector);
    return Array.from(ids).map((id) => new Element(this._raw, id));
  }

  // ── Tree ─────────────────────────────────────────────────────────────────

  /** Append `child` as the last child of `parent`. */
  appendChild(parent: Element, child: Element): void {
    this._raw.appendChild(parent._nodeId, child._nodeId);
  }

  /** Detach `node` from its parent. */
  removeChild(node: Element): void {
    this._raw.remove(node._nodeId);
  }
}
