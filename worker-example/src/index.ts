/**
 * Cloudflare Worker: Wikipedia page cleaner powered by linkedom-rust.
 *
 * Fetches a Wikipedia article, uses the Rust/WASM DOM to:
 *   1. Collect all image `src` URLs.
 *   2. Remove navigation chrome (nav, footer, sidebar, TOC, etc.).
 *   3. Return a JSON response with the image list and cleaned HTML.
 *
 * ## Prerequisites
 *
 * Before deploying, build the WASM package from the repo root:
 *
 *   ```sh
 *   cd js && npm run build:wasm
 *   ```
 *
 * Then deploy with wrangler from this directory:
 *
 *   ```sh
 *   wrangler deploy
 *   ```
 *
 * ⚠  LOCAL BLOCKER — As of current HEAD, the `wasm32-unknown-unknown` target is
 * not available on the development machine (Homebrew rustc, no rustup).  The
 * WASM binary cannot be produced until `rustup target add wasm32-unknown-unknown`
 * is run.  Cloudflare Workers runtime compatibility is not yet proven end-to-end.
 *
 * @module
 */

// Path is relative to the wasm-pack --out-dir (../pkg from repo root).
// wrangler resolves this at bundle time via the CompiledWasm rule in wrangler.toml.
import init, { WasmDocument } from '../../pkg/linkedom_rust.js';

// ── Types ─────────────────────────────────────────────────────────────────────

interface CleanedPage {
  url: string;
  images: string[];
  html: string;
}

// ── WASM initialisation ───────────────────────────────────────────────────────

// Top-level await is supported in ES-module Workers.
// The WASM binary is compiled once per Worker isolate lifetime.
let wasmReady: Promise<void> | null = null;

function ensureWasm(): Promise<void> {
  if (wasmReady === null) {
    wasmReady = init().catch((err) => {
      wasmReady = null;
      throw err;
    });
  }
  return wasmReady;
}

// ── DOM helpers ───────────────────────────────────────────────────────────────

/** Selectors for navigation chrome to strip from Wikipedia pages. */
const CHROME_SELECTORS = [
  'nav',
  'footer',
  // Wikipedia-specific
  '#mw-navigation',
  '#mw-head',
  '#mw-panel',
  '#mw-page-base',
  '#mw-head-base',
  '#catlinks',
  '#footer',
  '#toc',
  '.navbox',
  '.sidebar',
  '.mw-editsection',
  '.printfooter',
  '#siteNotice',
  '#contentSub',
];

/**
 * Parse `html`, strip navigation chrome, and return cleaned HTML plus image srcs.
 */
function cleanPage(html: string): { images: string[]; cleanedHtml: string } {
  const doc = WasmDocument.parseHtml(html);

  // Collect image srcs before removing elements that might contain them.
  const imgIds = doc.querySelectorAll('img');
  const images: string[] = [];
  for (let i = 0; i < imgIds.length; i++) {
    const src = doc.getAttribute(imgIds[i]!, 'src');
    if (src) images.push(src);
  }

  // Strip navigation chrome.
  for (const sel of CHROME_SELECTORS) {
    const ids = doc.querySelectorAll(sel);
    for (let i = 0; i < ids.length; i++) {
      doc.remove(ids[i]!);
    }
  }

  return { images, cleanedHtml: doc.serialize() };
}

// ── Request handler ───────────────────────────────────────────────────────────

/**
 * Parse the `page` query parameter into a Wikipedia article URL.
 * Falls back to the English Wikipedia main page.
 */
function resolveWikipediaUrl(requestUrl: string): string {
  const url = new URL(requestUrl);
  const page = url.searchParams.get('page') ?? 'Main_Page';
  // Sanitise: allow only word characters, underscores, hyphens, and spaces.
  const safePage = page.replace(/[^\w\s\-]/g, '').trim().replace(/\s+/g, '_');
  return `https://en.wikipedia.org/wiki/${encodeURIComponent(safePage)}`;
}

export default {
  async fetch(request: Request): Promise<Response> {
    // Only handle GET requests.
    if (request.method !== 'GET') {
      return new Response('Method Not Allowed', { status: 405 });
    }

    try {
      await ensureWasm();
    } catch (err) {
      return new Response(`WASM init failed: ${String(err)}`, { status: 500 });
    }

    const wikiUrl = resolveWikipediaUrl(request.url);

    let html: string;
    try {
      const upstream = await fetch(wikiUrl, {
        headers: { 'User-Agent': 'linkedom-rust-worker/0.1 (example)' },
      });
      if (!upstream.ok) {
        return new Response(`Upstream error ${upstream.status}`, { status: 502 });
      }
      html = await upstream.text();
    } catch (err) {
      return new Response(`Fetch failed: ${String(err)}`, { status: 502 });
    }

    let result: { images: string[]; cleanedHtml: string };
    try {
      result = cleanPage(html);
    } catch (err) {
      return new Response(`DOM processing failed: ${String(err)}`, { status: 500 });
    }

    const body: CleanedPage = {
      url: wikiUrl,
      images: result.images,
      html: result.cleanedHtml,
    };

    return new Response(JSON.stringify(body, null, 2), {
      headers: { 'Content-Type': 'application/json; charset=utf-8' },
    });
  },
};
