/**
 * Type stub for the wasm-pack-generated WASM glue module.
 *
 * This file is committed as a placeholder so that `npm run typecheck` works without
 * running `npm run build:wasm` first.  Running `npm run build:wasm` will regenerate
 * this file with complete, accurate declarations derived from `src/wasm.rs`.
 *
 * The shapes here mirror the `IRawDocument` interface declared in `js/index.ts`.
 * `Result<T, JsValue>` methods are typed as returning `T` — wasm-bindgen converts
 * the `Err` variant into a thrown JavaScript `Error`.
 */

export declare class WasmDocument {
  static parseHtml(html: string): WasmDocument;
  rootId(): number;
  serialize(): string;
  createElement(tag: string): number;
  createTextNode(text: string): number;
  appendChild(parent: number, child: number): void;
  remove(node: number): void;
  children(node: number): Uint32Array;
  querySelector(selector: string): number | undefined;
  querySelectorAll(selector: string): Uint32Array;
  getAttribute(node: number, name: string): string | undefined;
  setAttribute(node: number, name: string, value: string): void;
  removeAttribute(node: number, name: string): void;
  getTextContent(node: number): string | undefined;
  setTextContent(node: number, text: string): void;
  getInnerHtml(node: number): string | undefined;
  setInnerHtml(node: number, html: string): void;
  classListAdd(node: number, cls: string): void;
  classListRemove(node: number, cls: string): void;
  /** Returns `true`/`false`; throws for invalid node ids or non-element nodes. */
  classListContains(node: number, cls: string): boolean;
}

/** Initialise the WASM binary.  Must be awaited before using `WasmDocument`. */
export default function init(
  input?: string | URL | BufferSource | WebAssembly.Module,
): Promise<void>;
