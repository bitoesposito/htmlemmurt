// The parts of the browser runtime that need no DOM, so tests can run them in Node.

export const SCRIPT_TYPE = 'text/htmlemmurt';

/**
 * Compiles a page, fetching the files it includes. The compiler reads files
 * synchronously, so each round compiles with the files fetched so far and
 * fetches the one it was missing, until nothing is missing.
 *
 * @param {string} source
 * @param {string} filename  paths in the page are relative to it
 * @param {(path: string) => Promise<string>} load  rejects when the file is missing
 * @param {(source: string, options: object) => any} compile  the htmlemmurt compile()
 */
export async function compileWithFiles(source, filename, load, compile) {
  const files = new Map();
  for (;;) {
    const wanted = [];
    const result = compile(source, {
      filename,
      inline: true,
      read: (path) => {
        if (files.has(path)) return files.get(path);
        wanted.push(path);
        return null;
      },
    });
    if (wanted.length === 0) return result;
    await Promise.all(
      wanted.map(async (path) => {
        files.set(path, await load(path).catch(() => null));
      }),
    );
  }
}

/** Name of a page in errors: its `src`, or `pagina-N.emmurt` when inline. */
export function scriptName(src, index) {
  return src ? src.split(/[?#]/)[0] : `pagina-${index + 1}.emmurt`;
}
