import { test } from 'node:test';
import assert from 'node:assert/strict';
import { compile } from '../index.js';
import { compileWithFiles, scriptName } from '../lib/runtime.js';

const files = {
  'sito/pezzi/menu.emmurt': '<tomtom><chiamm da="voce.emmurt" /></tomtom>',
  'sito/pezzi/voce.emmurt': '<voce>Pizze</voce>',
  'sito/main.fratm': 'stamm a dì("Uè")',
};

function loader() {
  const fetched = [];
  const load = async (path) => {
    fetched.push(path);
    if (!(path in files)) throw new Error('404');
    return files[path];
  };
  return { load, fetched };
}

test('fetches nested includes until the page compiles', async () => {
  const { load, fetched } = loader();
  const source = '<chiamm da="pezzi/menu.emmurt" /><fratm da="main.fratm" />';
  const { html, errors } = await compileWithFiles(source, 'sito/index.emmurt', load, compile);
  assert.deepEqual(errors, []);
  assert.equal(html, '<nav><li>Pizze</li></nav><script>\nconsole.log("Uè");\n</script>');
  assert.deepEqual(fetched, ['sito/pezzi/menu.emmurt', 'sito/pezzi/voce.emmurt', 'sito/main.fratm']);
});

test('a file that cannot be fetched is a compile error', async () => {
  const { load } = loader();
  const { errors } = await compileWithFiles('<chiamm da="nada.emmurt" />', 'sito/index.emmurt', load, compile);
  assert.equal(errors[0].message, "Nun trovo 'nada.emmurt'.");
});

test('script names', () => {
  assert.equal(scriptName('pagine/index.emmurt?v=2', 0), 'pagine/index.emmurt');
  assert.equal(scriptName(null, 1), 'pagina-2.emmurt');
});
