import type { Plugin } from 'vite';

/**
 * Serves and builds every `x.emmurt` page as `x.html`. Pages are the .emmurt
 * files that start with `<cristian>`; set `build.rollupOptions.input` to
 * choose them yourself. Add `fratmscript/vite` for the `.fratm` modules.
 */
export default function htmlemmurt(): Plugin;
export { htmlemmurt };
