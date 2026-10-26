import htmlemmurt from 'htmlemmurt/vite';
import fratmscript from 'fratmscript/vite';

// htmlemmurt serves the .emmurt pages, FratmScript compiles the .fratm modules
export default {
  plugins: [htmlemmurt(), fratmscript()],
};
