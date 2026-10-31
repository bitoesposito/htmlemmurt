import htmlemmurt from 'htmlemmurt/vite';
import fratmscript from 'fratmscript/vite';

// htmlemmurt serves the .emmurt pages, FratmScript compiles the .fratm modules
export default {
  // Relative URLs, so the build also works under user.github.io/repo/
  base: './',
  plugins: [htmlemmurt(), fratmscript()],
};
