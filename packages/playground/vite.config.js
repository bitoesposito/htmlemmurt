import htmlemmurt from 'htmlemmurt/vite';
import fratmscript from 'fratmscript/vite';
import parocss from 'par-o-css/vite';

export default {
  // Relative URLs, so the build works under github.io/htmlemmurt/
  base: './',
  plugins: [htmlemmurt(), fratmscript(), parocss()],
  // The examples, guide and lessons are pages too: only index is the app
  // CodeMirror with the HTML, CSS and JS languages is most of the bundle
  build: { rollupOptions: { input: 'index.html' }, chunkSizeWarningLimit: 700 },
  // The compilers find their .wasm next to their module: do not pre-bundle them
  optimizeDeps: { exclude: ['htmlemmurt', 'par-o-css'] },
  // The examples come from the repository's examples/ folder
  server: { fs: { allow: ['../..'] } },
};
