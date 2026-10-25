import type { Plugin } from 'vite';

/**
 * Serves and builds every `x.emmurt` page as `x.html` and compiles `.fratm`
 * modules. Pages are the .emmurt files that start with `<cristian>`; set
 * `build.rollupOptions.input` to choose them yourself.
 */
export default function htmlemmurt(): Plugin;
export { htmlemmurt };
