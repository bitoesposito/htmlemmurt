// htmlemmurt compiler for browsers. Call `init()` once before the rest.
import initWasm, * as wasm from './wasm/web/emmurt_wasm.js';
import { createApi } from './lib/api.js';

const api = createApi(wasm);
let ready = null;
let loaded = false;

export function init(input) {
  ready ??= initWasm(input === undefined ? undefined : { module_or_path: input }).then(() => {
    loaded = true;
  });
  return ready;
}

function loadedApi() {
  if (!loaded) throw new Error('htmlemmurt/web: await init() before using the compiler');
  return api;
}

export const compile = (source, options) => loadedApi().compile(source, options);
export const compileFratm = (source, options) => loadedApi().compileFratm(source, options);
export const vocabulary = () => loadedApi().vocabulary();
export const version = () => loadedApi().version();
export const runtimeError = (error) => loadedApi().runtimeError(error);
export { formatError } from './lib/api.js';
