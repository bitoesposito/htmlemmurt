import htmlemmurt from 'htmlemmurt/vite';

export default {
  // Relative URLs, so the build works under github.io/htmlemmurt/
  base: './',
  plugins: [htmlemmurt()],
  // The examples in esempi/ are pages too: only index is the app
  // CodeMirror with the HTML, CSS and JS languages is most of the bundle
  build: { rollupOptions: { input: 'index.html' }, chunkSizeWarningLimit: 700 },
  // The compiler finds its .wasm next to its module: do not pre-bundle it
  optimizeDeps: { exclude: ['htmlemmurt'] },
  // The examples come from the repository's examples/ folder
  server: { fs: { allow: ['../..'] } },
};
