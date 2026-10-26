// Editor logic that needs no VS Code API, so tests can run it in Node.
'use strict';

const RAW = ['fratm', 'copione', 'cess'];

/**
 * What the cursor is in, from the text before it: a tag name, a closing tag,
 * an attribute name or an attribute value. Null in text and in raw elements.
 */
function context(before) {
  // Past the start tag of an unclosed <fratm>, <copione> or <cess>: code, not markup
  const lower = before.toLowerCase();
  for (const raw of RAW) {
    const start = lower.lastIndexOf(`<${raw}`);
    const end = lower.indexOf('>', start);
    if (start > lower.lastIndexOf(`</${raw}`) && end !== -1 && lower[end - 1] !== '/') return null;
  }
  const open = before.lastIndexOf('<');
  if (open === -1 || open < before.lastIndexOf('>')) return null;
  const inTag = before.slice(open + 1);
  let m;
  if ((m = /^\/([\p{L}\p{N}_-]*)$/u.exec(inTag))) return { kind: 'closing', prefix: m[1] };
  if ((m = /^([\p{L}\p{N}_-]*)$/u.exec(inTag))) return { kind: 'tag', prefix: m[1] };
  const tag = /^[\p{L}\p{N}_-]+/u.exec(inTag)?.[0];
  if (!tag) return null;
  // Drop complete quoted values: an opening quote left means we are in one
  const unquoted = inTag.replace(/"[^"]*"|'[^']*'/g, '');
  if (/["']/.test(unquoted)) {
    m = /([^\s"'=<>/]+)\s*=\s*["']([^"']*)$/.exec(inTag);
    return m ? { kind: 'value', tag: tag.toLowerCase(), attr: m[1].toLowerCase(), prefix: m[2] } : null;
  }
  m = /\s([^\s"'=<>/]*)$/.exec(inTag);
  return m ? { kind: 'attr', tag: tag.toLowerCase(), prefix: m[1] } : null;
}

function htmlOf(vocab, tag) {
  return vocab.tags.find(([nap]) => nap === tag)?.[1];
}

/** The attribute a Neapolitan name means on an element, as the compiler picks it. */
function attrFor(vocab, nap, element) {
  return vocab.attributes.find((a) => a.nap === nap && (!a.only || a.only.includes(element)));
}

function tagItems(vocab) {
  return [
    ...vocab.tags.map(([nap, html]) => ({ label: nap, detail: `<${html}>` })),
    { label: 'chiamm', detail: 'ncolla n\'atu file .emmurt' },
  ];
}

function attrItems(vocab, tag) {
  const element = htmlOf(vocab, tag) ?? tag;
  const seen = new Set();
  const items = [];
  for (const a of vocab.attributes) {
    if (seen.has(a.nap) || (a.only && !a.only.includes(element))) continue;
    seen.add(a.nap);
    items.push({ label: a.nap, detail: a.html });
  }
  for (const [nap, html] of vocab.events) items.push({ label: nap, detail: `${html} (FratmScript)` });
  return items;
}

function valueItems(vocab, tag, attr) {
  const found = attrFor(vocab, attr, htmlOf(vocab, tag) ?? tag);
  return (found?.values ?? []).map(([nap, html]) => ({ label: nap, detail: html }));
}

/** What a word means in HTML, for the hover. */
function meaning(vocab, word) {
  const lower = word.toLowerCase();
  const lines = [];
  const html = htmlOf(vocab, lower);
  if (html) lines.push(`<${lower}> → <${html}>`);
  if (lower === 'chiamm') lines.push('<chiamm da="…" /> → ncolla n\'atu file .emmurt quanno compile');
  for (const a of vocab.attributes.filter((a) => a.nap === lower)) {
    lines.push(`${lower} → ${a.html}${a.only ? ` (ncopp'a <${a.only.join('>, <')}>)` : ''}`);
  }
  const event = vocab.events.find(([nap]) => nap === lower);
  if (event) lines.push(`${lower} → ${event[1]} (dint'ê virgulette ce va FratmScript)`);
  return lines;
}

/** Where to show a compiler error in the document `file`: 0-based. */
function errorPlace(error, file) {
  const same = error.file.replace(/\\/g, '/') === file.replace(/\\/g, '/');
  if (same) return { line: error.line - 1, column: error.column - 1, message: errorText(error) };
  // An error in an included file: show it at the top, naming the file
  return { line: 0, column: 0, message: `${error.file}:${error.line}:${error.column}: ${errorText(error)}` };
}

function errorText(error) {
  return error.suggestion ? `${error.message}\n${error.suggestion}` : error.message;
}

module.exports = { context, tagItems, attrItems, valueItems, meaning, errorPlace };
