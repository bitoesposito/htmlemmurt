import htmlemmurt from 'htmlemmurt/vite';
import fratmscript from 'fratmscript/vite';

// htmlemmurt serves the .emmurt pages, FratmScript compiles the .fratm modules
// and answers /api/… on the dev server with the Hono app in server/api.fratm
export default {
  plugins: [htmlemmurt(), fratmscript({ api: 'server/api.fratm' })],
};
