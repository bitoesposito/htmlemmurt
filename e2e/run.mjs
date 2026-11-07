// End-to-end checks in Chromium: the pizzeria example through the Vite dev
// server and its production build, the browser runtime without a build and
// the playground. Needs packages/htmlemmurt built, examples/pizzeria
// installed and packages/playground built.
// CHROMIUM_PATH picks a Chromium other than Playwright's.
import { spawn, execFileSync } from 'node:child_process';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';
import { chromium } from 'playwright';

const repo = join(dirname(fileURLToPath(import.meta.url)), '..');
const pizzeria = join(repo, 'examples', 'pizzeria');
const vite = join(pizzeria, 'node_modules', '.bin', 'vite');
const servers = [];

async function serve(command, args, cwd, url) {
  const child = spawn(command, args, { cwd, stdio: 'ignore' });
  servers.push(child);
  for (let i = 0; i < 100; i++) {
    try {
      // The base, to append paths that start with /
      if ((await fetch(url)).status < 500) return url.replace(/\/$/, '');
    } catch {}
    await new Promise((r) => setTimeout(r, 100));
  }
  throw new Error(`${command} did not start on ${url}`);
}

/** A page that fails the run on console errors (except a missing favicon). */
async function open(browser, url) {
  const page = await browser.newPage();
  page.problems = [];
  page.on('console', (m) => {
    if (m.type() === 'error' && !m.location().url.endsWith('favicon.ico')) page.problems.push(m.text());
  });
  page.on('pageerror', (e) => page.problems.push(e.message));
  await page.goto(url);
  return page;
}

async function pizzeriaWorks(browser, base) {
  const page = await open(browser, base + '/');
  assert.equal(await page.title(), "'A Pizzeria 'e Gennaro");
  await page.click('text=Sforna na pizza');
  await page.click('text=Sforna na pizza');
  assert.equal(await page.textContent('#sfurnate'), '2');

  await page.click('nav >> text=Menù');
  await page.waitForURL(/menu/);
  const add = page.locator('[data-pizza]');
  await add.nth(0).click();
  await add.nth(2).click();
  assert.equal(await page.textContent('#quante'), '2');

  // The cart is in localStorage: it survives a page change
  await page.goto(base + '/');
  assert.equal(await page.textContent('#quante'), '2');

  await page.goto(base + '/menu');
  await page.fill('#nomme', 'Gennaro');
  await page.fill('#telefono', '081 123');
  await page.click("text=Manna l'ordine");
  assert.equal(await page.textContent('#risposta'), 'Grazie Gennaro! 2 pizze stanno arrivanno.');
  assert.equal(await page.textContent('#quante'), '0');
  assert.deepEqual(page.problems, []);
  await page.close();
}

async function runtimeWorks(browser, base) {
  // A whole page replaces the document
  let page = await open(browser, base + '/examples/senza-build/');
  await page.waitForSelector('#saluto');
  assert.equal(await page.title(), 'Senza build');
  assert.equal(await page.evaluate(() => document.doctype?.name), 'html');
  await page.click('text=Tuocca');
  await page.click('text=Tuocca');
  assert.equal(await page.textContent('#tuccate'), '2');
  assert.deepEqual(page.problems, []);
  await page.close();

  // Fragments take the place of their tag; errors show in the page and the console
  page = await open(browser, base + '/e2e/frammento.html');
  await page.waitForSelector('pre');
  assert.equal(await page.textContent('#x'), 'Uè!');
  assert.match(await page.textContent('pre'), /<div> nun è napoletano/);
  assert.ok(page.problems.some((p) => p.includes('Vuoi dire <cascett>?')), page.problems.join('\n'));
  await page.close();
}

async function playgroundWorks(browser, base) {
  const page = await open(browser, base + '/');
  const anteprima = page.frameLocator('#anteprima iframe');
  // The status bar says how things are without words: pronto, sbaglio, attesa, carica
  const stato = (s) => page.locator(`#barra-stato[data-stato="${s}"]`).waitFor();
  await stato('pronto');
  assert.equal(await anteprima.locator('h1').textContent(), 'Uè, frà!');
  await anteprima.locator('button').click();
  await page.locator('#console li', { hasText: 'Driiin! 1' }).waitFor();
  await page.locator('#html .cm-content', { hasText: '<!DOCTYPE html>' }).waitFor();

  // The vocabulary, behind the ? button
  await page.click('#aiuto-apri');
  await page.fill('#cerca', 'cascett');
  assert.ok((await page.locator('#parole tr:visible').allTextContents()).some((r) => r.includes('<div>')));
  await page.click('#aiuto-chiude');

  // A mistake on purpose: the error in Neapolitan, also under the code
  await page.selectOption('#esempi', 'sbagliato');
  await stato('sbaglio');
  assert.match(await page.locator('#sbagli').textContent(), /<div> nun è napoletano\.\s*Vuoi dire <cascett>\?/);
  await page.locator('.cm-lintRange-error').first().waitFor();

  // The editor completes with Neapolitan names, never English HTML
  await page.click('#editor .cm-content');
  await page.keyboard.press('Control+End');
  await page.keyboard.type('\n<cas');
  await page.locator('.cm-tooltip-autocomplete li').first().waitFor();
  const proposte = await page.locator('.cm-tooltip-autocomplete li').allTextContents();
  assert.ok(proposte.some((p) => p.startsWith('cascett')) && !proposte.some((p) => p.startsWith('canvas')), proposte.join());
  await page.keyboard.press('Escape');

  // Format: only spaces and line breaks move
  await page.selectOption('#esempi', 'munno');
  await stato('pronto');
  await page.click('#editor .cm-content');
  await page.keyboard.press('Control+Home');
  await page.keyboard.type('   ');
  await page.click('#formatta');
  await page.locator('#editor .cm-line').first().filter({ hasText: /^<!--/ }).waitFor();

  // Learn: the solution of the first lesson marks it done
  await page.click('#impara');
  await page.locator('#lezione:not([hidden])').waitFor();
  await page.click('#lezione-soluzione');
  await page.locator('#lezione-compito[data-fatto]').waitFor();
  await page.click('#lezione-esci');

  // The guide: every snippet runs in its own preview and opens in the editor
  await page.click('#guida');
  await page.frameLocator('#guida-esempi iframe').first().locator('h1').waitFor();
  await page.click('#guida-esempi [data-azione="apri"]');
  await page.locator('#guida-paggena').waitFor({ state: 'hidden' });
  await stato('pronto');

  // The shared link opens the same code
  await page.selectOption('#esempi', 'modulo');
  await stato('pronto');
  await page.click('#condividi');
  await page.waitForFunction(() => location.hash.startsWith('#codice='));
  const link = page.url();
  // The sandboxed preview may not autofocus (subbeto in "Nu modulo"): Chrome says so
  assert.deepEqual(page.problems.filter((p) => !p.startsWith('Blocked autofocusing')), []);
  await page.close();

  const shared = await open(browser, link);
  await shared.locator('#barra-stato[data-stato="pronto"]').waitFor();
  await shared.frameLocator('#anteprima iframe').locator('form').waitFor();
  // Once open, the code lives in localStorage: a reload after an edit keeps the edit
  await shared.click('#editor .cm-content');
  await shared.keyboard.type(' ');
  assert.equal(await shared.evaluate(() => location.hash), '');
  assert.deepEqual(shared.problems.filter((p) => !p.startsWith('Blocked autofocusing')), []);
  await shared.close();
}

const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH });
try {
  const dev = await serve(vite, ['--port', '5191', '--strictPort'], pizzeria, 'http://localhost:5191/');
  await pizzeriaWorks(browser, dev);
  console.log('✓ pizzeria, dev server');

  execFileSync(vite, ['build', '--logLevel', 'warn'], { cwd: pizzeria, stdio: 'inherit' });
  const preview = await serve(vite, ['preview', '--port', '5192', '--strictPort'], pizzeria, 'http://localhost:5192/');
  await pizzeriaWorks(browser, preview);
  console.log('✓ pizzeria, production build');

  const files = await serve('python3', ['-m', 'http.server', '5193'], repo, 'http://localhost:5193/');
  await runtimeWorks(browser, files);
  console.log('✓ browser runtime');

  const playground = join(repo, 'packages', 'playground', 'dist');
  const site = await serve('python3', ['-m', 'http.server', '5194'], playground, 'http://localhost:5194/');
  await playgroundWorks(browser, site);
  console.log('✓ playground');
} finally {
  await browser.close();
  for (const server of servers) server.kill();
}
