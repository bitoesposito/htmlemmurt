// Runs htmlemmurt in a web page, no build step:
//
//   <script type="module" src="…/htmlemmurt/browser.js"></script>
//   <script type="text/htmlemmurt" src="index.emmurt"></script>
//
// Each <script type="text/htmlemmurt"> (inline or with src) is compiled and
// takes the place of the tag; a whole page (<paggena>) replaces the document.
// Errors are shown in the page and in the console, in Neapolitan. `ready`
// resolves when every script has been shown.
import { init, compile, runtimeError, formatError } from './web.js';
import { SCRIPT_TYPE, compileWithFiles, scriptName } from './lib/runtime.js';

async function load(path) {
  const response = await fetch(new URL(path, document.baseURI));
  if (!response.ok) throw new Error(`${response.status} ${response.statusText}`);
  return response.text();
}

function domReady() {
  if (document.readyState !== 'loading') return Promise.resolve();
  return new Promise((resolve) => document.addEventListener('DOMContentLoaded', resolve, { once: true }));
}

function report(script, message) {
  console.error(`🤌 htmlemmurt ${message}`);
  const box = document.createElement('pre');
  box.style.cssText = 'color:#7a1f1f;background:#fff4f4;padding:1rem;border-left:4px solid #c0392b;white-space:pre-wrap';
  box.textContent = `Uh, mannaggia! ${message}`;
  script.replaceWith(box);
}

function show(html, script) {
  if (html.startsWith('<!DOCTYPE html>')) {
    document.open();
    document.write(html);
    document.close();
    return;
  }
  const template = document.createElement('template');
  template.innerHTML = html;
  const scripts = [...template.content.querySelectorAll('script')];
  script.replaceWith(template.content);
  // Scripts parsed by innerHTML never run: swap each for a fresh copy
  for (const old of scripts) {
    const fresh = document.createElement('script');
    for (const { name, value } of old.attributes) fresh.setAttribute(name, value);
    fresh.textContent = old.textContent;
    old.replaceWith(fresh);
  }
}

window.addEventListener('error', (event) => {
  if (event.error instanceof Error) console.error(`🤌 ${runtimeError(event.error)}`);
});
window.addEventListener('unhandledrejection', (event) => {
  if (event.reason instanceof Error) console.error(`🤌 ${runtimeError(event.reason)}`);
});

export const ready = Promise.all([init(), domReady()]).then(async () => {
  const scripts = [...document.querySelectorAll(`script[type="${SCRIPT_TYPE}"]`)];
  for (const [index, script] of scripts.entries()) {
    const src = script.getAttribute('src');
    const filename = scriptName(src, index);
    let source;
    try {
      source = src ? await load(src) : script.textContent;
    } catch (error) {
      report(script, `nun riesco a caricà ${filename} (${error.message})`);
      continue;
    }
    const { html, errors } = await compileWithFiles(source, filename, load, compile);
    if (errors.length > 0) report(script, formatError(errors[0]));
    else show(html, script);
  }
});
