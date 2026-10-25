// End-to-end checks in Chromium: the pizzeria example through the Vite dev
// server and its production build, and the browser runtime without a build.
// Needs packages/htmlemmurt built and examples/pizzeria installed.
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
      if ((await fetch(url)).status < 500) return url;
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

const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH });
try {
  const dev = await serve(vite, ['--port', '5191', '--strictPort'], pizzeria, 'http://localhost:5191/');
  await pizzeriaWorks(browser, dev.slice(0, -1));
  console.log('✓ pizzeria, dev server');

  execFileSync(vite, ['build', '--logLevel', 'warn'], { cwd: pizzeria, stdio: 'inherit' });
  const preview = await serve(vite, ['preview', '--port', '5192', '--strictPort'], pizzeria, 'http://localhost:5192/');
  await pizzeriaWorks(browser, preview.slice(0, -1));
  console.log('✓ pizzeria, production build');

  const files = await serve('python3', ['-m', 'http.server', '5193'], repo, 'http://localhost:5193/');
  await runtimeWorks(browser, files.slice(0, -1));
  console.log('✓ browser runtime');
} finally {
  await browser.close();
  for (const server of servers) server.kill();
}
