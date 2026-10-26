// htmlemmurt compiler for Node.js (and bundlers that run in Node).
import { existsSync, readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { createApi } from './lib/api.js';

const wasm = createRequire(import.meta.url)('./wasm/node/emmurt_wasm.js');
const api = createApi(wasm);

export const { vocabulary, version, runtimeError } = api;
export { formatError } from './lib/api.js';

/**
 * Compiles a page. Included files are read from disk: paths starting with
 * `/` that are not absolute paths on disk are taken from `root`.
 */
export function compile(source, options = {}) {
  const { root = process.cwd(), read = (path) => readFrom(root, path), ...rest } = options;
  return api.compile(source, { ...rest, read });
}

function readFrom(root, path) {
  const file = path.startsWith('/') && !existsSync(path) ? join(root, path) : path;
  return existsSync(file) ? readFileSync(file, 'utf8') : null;
}
