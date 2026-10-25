'use strict';
const { test } = require('node:test');
const assert = require('node:assert/strict');
const { existsSync } = require('node:fs');
const { join } = require('node:path');
const lingua = require('../lib/lingua.js');

const vocab = {
  tags: [
    ['cascetta', 'div'],
    ['campo', 'input'],
    ['scamazza', 'button'],
  ],
  attributes: [
    { nap: 'tipo', html: 'type', only: ['input'], values: [['segreto', 'password']] },
    { nap: 'tipo', html: 'type', only: ['button'], values: [['buttone', 'button']] },
    { nap: 'classe', html: 'class' },
    { nap: 'tipo', html: 'type' },
  ],
  events: [['quanno-tuocche', 'onclick']],
};

test('context: where the cursor is', () => {
  assert.deepEqual(lingua.context('<cas'), { kind: 'tag', prefix: 'cas' });
  assert.deepEqual(lingua.context('<cascetta></cas'), { kind: 'closing', prefix: 'cas' });
  assert.deepEqual(lingua.context('<cascetta cl'), { kind: 'attr', tag: 'cascetta', prefix: 'cl' });
  assert.deepEqual(lingua.context('<campo classe="a b" '), { kind: 'attr', tag: 'campo', prefix: '' });
  assert.deepEqual(lingua.context('<campo tipo="se'), { kind: 'value', tag: 'campo', attr: 'tipo', prefix: 'se' });
  assert.deepEqual(lingua.context('<campo\n  tipo="'), { kind: 'value', tag: 'campo', attr: 'tipo', prefix: '' });
  assert.equal(lingua.context('<cascetta>testo'), null);
  assert.equal(lingua.context('testo'), null);
  assert.equal(lingua.context('<fratm>\n si (a <'), null);
  assert.deepEqual(lingua.context('<fratm>x</fratm><cas'), { kind: 'tag', prefix: 'cas' });
  assert.deepEqual(lingua.context('<fratm da="x" /><cas'), { kind: 'tag', prefix: 'cas' });
});

test('items follow the element, like the compiler', () => {
  assert.deepEqual(lingua.tagItems(vocab).map((i) => i.label), ['cascetta', 'campo', 'scamazza', 'chiamm']);
  const attrs = lingua.attrItems(vocab, 'campo');
  assert.deepEqual(attrs.map((i) => i.label), ['tipo', 'classe', 'quanno-tuocche']);
  assert.deepEqual(lingua.valueItems(vocab, 'campo', 'tipo'), [{ label: 'segreto', detail: 'password' }]);
  assert.deepEqual(lingua.valueItems(vocab, 'scamazza', 'tipo'), [{ label: 'buttone', detail: 'button' }]);
  assert.deepEqual(lingua.valueItems(vocab, 'cascetta', 'tipo'), []);
});

test('meaning for the hover', () => {
  assert.deepEqual(lingua.meaning(vocab, 'cascetta'), ['<cascetta> → <div>']);
  assert.deepEqual(lingua.meaning(vocab, 'quanno-tuocche'), ["quanno-tuocche → onclick (dint'ê virgulette ce va FratmScript)"]);
  assert.deepEqual(lingua.meaning(vocab, 'pizza'), []);
});

test('errors in included files show at the top', () => {
  const error = { file: '/a/pezzi/b.emmurt', line: 3, column: 4, message: 'Uh', suggestion: 'Fa accussì' };
  assert.deepEqual(lingua.errorPlace(error, '/a/pezzi/b.emmurt'), { line: 2, column: 3, message: 'Uh\nFa accussì' });
  assert.equal(lingua.errorPlace(error, '/a/index.emmurt').message, '/a/pezzi/b.emmurt:3:4: Uh\nFa accussì');
});

test('the shipped compiler works', { skip: !existsSync(join(__dirname, '..', 'compiler', 'emmurt_wasm.js')) }, () => {
  const wasm = require('../compiler/emmurt_wasm.js');
  const vocabulary = JSON.parse(wasm.vocabulary());
  assert.ok(vocabulary.tags.length > 50);
  const { errors } = wasm.compile('<div>', 'a.emmurt', () => null, true);
  assert.equal(errors[0].suggestion, 'Vuoi dire <cascetta>?');
});
