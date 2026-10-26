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

/** A page that fails the run on console errors (but a missing favicon). */
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
  const stato = page.locator('#stato');
  const anteprima = page.frameLocator('#anteprima iframe');
  await page.waitForFunction(() => document.querySelector('#stato').textContent.includes('Tutto a posto'));
  assert.equal(await anteprima.locator('h1').textContent(), 'Uè, frà!');
  await anteprima.locator('button').click();
  await page.locator('#console li', { hasText: 'Driiin! 1' }).waitFor();

  await page.click('#scheda-html');
  await page.locator('#html .cm-content', { hasText: '<!DOCTYPE html>' }).waitFor();
  await page.click('#scheda-vocabolario');
  await page.fill('#cerca', 'cascett');
  assert.ok((await page.locator('#parole tr:visible').allTextContents()).some((r) => r.includes('<div>')));
  await page.click('#scheda-anteprima');

  // A mistake on purpose: the error in Neapolitan, also under the code
  await page.selectOption('#esempi', { label: 'Nu sbaglio, apposta' });
  await page.waitForFunction(() => document.querySelector('#stato').textContent.includes('nun è napoletano'));
  assert.match(await stato.textContent(), /Vuoi dire <cascett>\?/);
  await page.locator('.cm-lintRange-error').first().waitFor();

  // The shared link opens the same code
  await page.selectOption('#esempi', { label: 'Nu modulo' });
  await page.waitForFunction(() => document.querySelector('#stato').textContent.includes('Tutto a posto'));
  await page.click('#condividi');
  await page.waitForFunction(() => location.hash.startsWith('#codice='));
  const link = page.url();
  // The sandboxed preview may not autofocus (subbeto in "Nu modulo"): Chrome says so
  assert.deepEqual(page.problems.filter((p) => !p.startsWith('Blocked autofocusing')), []);
  await page.close();

  const shared = await open(browser, link);
  await shared.waitForFunction(() => document.querySelector('#stato').textContent.includes('Tutto a posto'));
  await shared.frameLocator('#anteprima iframe').locator('form').waitFor();
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
