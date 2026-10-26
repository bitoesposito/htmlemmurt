// Turns the raw responses of the WASM bindings into the public API.

/** @param {any} wasm */
export function createApi(wasm) {
  function compile(source, options = {}) {
    const { filename = 'input.emmurt', read, inline = false } = options;
    // The bindings take a missing file as anything that is not a string
    const response = wasm.compile(String(source), filename, read ? (path) => read(path) : undefined, inline);
    return { html: response.html, dependencies: response.dependencies, errors: response.errors };
  }

  function vocabulary() {
    return JSON.parse(wasm.vocabulary());
  }

  function runtimeError(error) {
    const name = error && error.name ? String(error.name) : 'Error';
    const message = error && error.message !== undefined ? String(error.message) : String(error);
    return wasm.runtimeError(name, message);
  }

  return { compile, vocabulary, version: wasm.version, runtimeError };
}

/** `file:line:column: message`, then the hint on its own line. */
export function formatError(error) {
  const hint = error.suggestion ? `\n  ${error.suggestion}` : '';
  return `${error.file}:${error.line}:${error.column}: ${error.message}${hint}`;
}
