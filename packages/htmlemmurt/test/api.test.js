import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, mkdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { compile, compileFratm, vocabulary, version, runtimeError, formatError } from '../index.js';

test('compile translates a page', () => {
  const { html, errors } = compile('<cristian><panz><cascett classe="a">Uè</cascett></panz></cristian>');
  assert.deepEqual(errors, []);
  assert.equal(html, '<!DOCTYPE html>\n<html><body><div class="a">Uè</div></body></html>');
});

test('compile reports errors in Neapolitan, never throws', () => {
  const { html, errors } = compile('<div>', { filename: 'index.emmurt' });
  assert.equal(html, null);
  assert.deepEqual(errors, [
    {
      message: '<div> nun è napoletano.',
      file: 'index.emmurt',
      line: 1,
      column: 2,
      suggestion: 'Vuoi dire <cascett>?',
    },
  ]);
  assert.equal(formatError(errors[0]), 'index.emmurt:1:2: <div> nun è napoletano.\n  Vuoi dire <cascett>?');
});

test('compile reads includes from disk, relative to the page', () => {
  const dir = mkdtempSync(join(tmpdir(), 'emmurt-'));
  mkdirSync(join(dir, 'pezzi'));
  writeFileSync(join(dir, 'pezzi', 'pere.emmurt'), '<pere>Uè</pere>\n');
  writeFileSync(join(dir, 'main.fratm'), 'stamm a dì(1)');
  const page = join(dir, 'index.emmurt');
  const source = '<chiamm da="pezzi/pere.emmurt" />\n<fratm da="/main.fratm" />';
  const { html, dependencies, errors } = compile(source, { filename: page, root: dir, inline: true });
  assert.deepEqual(errors, []);
  assert.equal(html, '<footer>Uè</footer>\n<script>\nconsole.log(1);\n</script>');
  assert.deepEqual(dependencies, [join(dir, 'pezzi', 'pere.emmurt'), '/main.fratm']);
});

test('compile takes a custom read function', () => {
  const files = { 'pezzi/a.emmurt': '<scatulell>a</scatulell>' };
  const read = (path) => files[path] ?? null;
  assert.equal(compile('<chiamm da="pezzi/a.emmurt" />', { read }).html, '<span>a</span>');
  const { errors } = compile('<chiamm da="nada.emmurt" />', { read });
  assert.equal(errors[0].message, "Nun trovo 'nada.emmurt'.");
  assert.equal(errors[0].suggestion, "L'aggio cercato ccà: nada.emmurt");
});

test('compileFratm compiles modules, with a source map', () => {
  const { code, map } = compileFratm('mann for facc f() { piglie 1 }', { filename: 'src/f.fratm' });
  assert.equal(code, 'export function f() {\n  return 1;\n}\n');
  assert.deepEqual([map.version, map.sources, map.file], [3, ['src/f.fratm'], 'src/f.js']);
  assert.ok(map.mappings.length > 0);
  const broken = compileFratm('chist x = 1', { filename: 'a.fratm' });
  assert.deepEqual([broken.code, broken.map], [null, null]);
  assert.equal(broken.errors[0].file, 'a.fratm');
});

test('vocabulary, version and runtime errors', () => {
  assert.deepEqual(vocabulary().tags[0], ['cristian', 'html']);
  assert.equal(vocabulary().attributes.find((a) => a.nap === 'classe').html, 'class');
  assert.match(version(), /^\d+\.\d+\.\d+$/);
  assert.match(runtimeError(new ReferenceError('x is not defined')), /x/);
});
