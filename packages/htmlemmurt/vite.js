// Vite plugin: .emmurt pages are the app's HTML entries, .fratm files its
// modules. `x.emmurt` is served and built as `x.html`.
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { join, relative, resolve, sep } from 'node:path';
import { compile, compileFratm, formatError } from './index.js';

const FRATM = /\.fratm$/;
// A request for a .fratm file that Vite would serve as it is
const RAW_FRATM_REQUEST = /^[^?#]*\.fratm(?:$|[?#])/;
// A page starts with <paggena>, maybe after comments
const PAGE = /^\s*(?:<!--[\s\S]*?-->\s*)*<paggena[\s>/]/i;
const SKIP_DIRS = new Set(['node_modules', 'dist', 'public']);

export default function htmlemmurt() {
  let root = process.cwd();

  /** The .emmurt source of an .html path that is not on disk. */
  const sourceOf = (htmlPath) => {
    if (!htmlPath.endsWith('.html') || existsSync(htmlPath)) return null;
    const source = htmlPath.slice(0, -'.html'.length) + '.emmurt';
    return existsSync(source) ? source : null;
  };

  const compilePage = (file) =>
    compile(readFileSync(file, 'utf8'), { filename: file, root });

  return {
    name: 'htmlemmurt',
    enforce: 'pre',

    config(config) {
      root = resolve(config.root ?? process.cwd());
      if (config.build?.rollupOptions?.input) return;
      const pages = findPages(root);
      if (pages.length === 0) return;
      const input = Object.fromEntries(
        pages.map((page) => [
          relative(root, page).slice(0, -'.emmurt'.length).split(sep).join('/'),
          page.slice(0, -'.emmurt'.length) + '.html',
        ]),
      );
      return { build: { rollupOptions: { input } } };
    },

    configResolved(config) {
      root = config.root;
    },

    resolveId(id) {
      return sourceOf(id) ? id : null;
    },

    load(id) {
      const file = sourceOf(id);
      if (!file) return null;
      const { html, dependencies, errors } = compilePage(file);
      if (errors.length > 0) this.error(errorFor(errors[0]));
      for (const dependency of dependencies) this.addWatchFile(resolve(root, dependency));
      return html;
    },

    transform(source, id) {
      const file = id.split('?', 1)[0];
      if (!FRATM.test(file)) return null;
      const { code, map, errors } = compileFratm(source, { filename: file });
      if (errors.length > 0) this.error(errorFor(errors[0]));
      return { code, map };
    },

    configureServer(server) {
      const reload = (file) => {
        if (file.endsWith('.emmurt')) server.ws.send({ type: 'full-reload' });
      };
      server.watcher.on('change', reload);
      server.watcher.on('add', reload);
      server.watcher.on('unlink', reload);

      server.middlewares.use(async (req, res, next) => {
        if (!req.url) return next();
        // The dev server only compiles requests it knows are modules. Imports
        // between modules get `?import`, the <script src> of a page does not
        if (RAW_FRATM_REQUEST.test(req.url) && !/[?&]import(?:[=&#]|$)/.test(req.url)) {
          req.url += req.url.includes('?') ? '&import' : '?import';
          return next();
        }
        if (req.method !== 'GET' && req.method !== 'HEAD') return next();
        let path = decodeURIComponent(req.url.split(/[?#]/, 1)[0]);
        if (path.endsWith('/')) path += 'index.html';
        else if (!/\.[^/]*$/.test(path)) path += '.html';
        const file = sourceOf(join(root, path));
        if (!file) return next();
        res.setHeader('Content-Type', 'text/html; charset=utf-8');
        const { html, errors } = compilePage(file);
        if (errors.length > 0) {
          const message = formatError({ ...errors[0], file: relative(root, errors[0].file) });
          server.config.logger.error(`🤌 ${message}`);
          res.statusCode = 500;
          res.end(errorPage(message));
          return;
        }
        try {
          res.end(await server.transformIndexHtml(path, html, req.originalUrl));
        } catch (error) {
          next(error);
        }
      });
    },
  };
}

/** Every .emmurt file under `dir` that is a page, not a piece for <chiamm>. */
function findPages(dir) {
  const pages = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) {
      if (!entry.name.startsWith('.') && !SKIP_DIRS.has(entry.name)) pages.push(...findPages(path));
    } else if (entry.name.endsWith('.emmurt') && PAGE.test(readFileSync(path, 'utf8'))) {
      pages.push(path);
    }
  }
  return pages.sort();
}

function errorFor(error) {
  return {
    message: error.suggestion ? `${error.message}\n${error.suggestion}` : error.message,
    id: error.file,
    loc: { file: error.file, line: error.line, column: error.column - 1 },
  };
}

// The Vite client reloads the page when the file is saved again
function errorPage(message) {
  const escaped = message.replace(/&/g, '&amp;').replace(/</g, '&lt;');
  return `<!DOCTYPE html>
<meta charset="utf-8">
<title>Uh, mannaggia</title>
<script type="module" src="/@vite/client"></script>
<style>body{font:16px/1.5 system-ui;margin:2rem;color:#7a1f1f}pre{background:#fff4f4;padding:1rem;border-left:4px solid #c0392b;white-space:pre-wrap}</style>
<h1>Uh, mannaggia! 🤌</h1>
<pre>${escaped}</pre>
<p>Aggiusta 'o file e salva: 'a paggena se ricarica sola.</p>
`;
}

export { htmlemmurt };
