/* Headless-only review. Pass a locally installed playwright-core and Chromium. */
'use strict';
const fs = require('node:fs');
const path = require('node:path');
const http = require('node:http');
const assert = require('node:assert/strict');
const [sitePath, outputPath, playwrightPath, executablePath] = process.argv.slice(2);
if (!executablePath) throw Error('Usage: node test_site_browser.cjs SITE OUTPUT PLAYWRIGHT_CORE CHROMIUM');
const { chromium } = require(path.resolve(playwrightPath));
const types = { '.html': 'text/html', '.css': 'text/css', '.js': 'text/javascript', '.json': 'application/json', '.svg': 'image/svg+xml', '.png': 'image/png' };
fs.mkdirSync(outputPath, { recursive: false });
const server = http.createServer((req, res) => {
  const url = new URL(req.url, 'http://127.0.0.1');
  const leaf = url.pathname.replace(/^\/dlpad\//, '') || 'index.html';
  if (!url.pathname.startsWith('/dlpad/') || leaf.includes('..') || !/^[a-zA-Z0-9_./-]+$/.test(leaf)) { res.writeHead(404).end(); return; }
  const file = path.join(sitePath, leaf);
  fs.readFile(file, (error, body) => {
    if (error) { res.writeHead(404).end(); return; }
    res.writeHead(200, { 'Content-Type': types[path.extname(file)] || 'application/octet-stream' }); res.end(body);
  });
});
(async () => {
  let browser;
  const checks = [];
  try {
    await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
    const base = `http://127.0.0.1:${server.address().port}/dlpad/`;
    browser = await chromium.launch({ executablePath, headless: true });
    for (const [name, width, height, colorScheme] of [['desktop-light', 1440, 1000, 'light'], ['tablet-light', 768, 1024, 'light'], ['mobile-light', 360, 800, 'light'], ['desktop-dark', 1440, 1000, 'dark'], ['mobile-dark', 360, 800, 'dark'], ['zoom-200', 720, 500, 'light']]) {
      const context = await browser.newContext({ viewport: { width, height }, deviceScaleFactor: name === 'zoom-200' ? 2 : 1, colorScheme, reducedMotion: 'reduce' });
      const page = await context.newPage();
      const errors = []; page.on('pageerror', e => errors.push(e.message));
      await page.goto(base); await page.waitForFunction(() => document.getElementById('release-state').textContent.includes('첫 공개'));
      assert.equal(await page.locator('html').getAttribute('data-theme'), colorScheme);
      assert.equal(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth), false, `${name}: horizontal overflow`);
      assert.equal(await page.locator('img').evaluateAll(imgs => imgs.every(img => img.complete && img.naturalWidth > 0)), true);
      await page.keyboard.press('Tab'); await page.keyboard.press('Enter');
      assert.equal(await page.evaluate(() => document.activeElement.id), 'main');
      await page.getByRole('button', { name: /테마로 변경/ }).click();
      assert.notEqual(await page.locator('html').getAttribute('data-theme'), colorScheme);
      await page.getByRole('button', { name: /테마로 변경/ }).click();
      await page.locator('summary').first().focus(); await page.keyboard.press('Enter');
      assert.equal(await page.locator('details').first().getAttribute('open'), '');
      await page.keyboard.press('Enter');
      const primary = page.locator('.primary'); const before = await primary.evaluate(e => getComputedStyle(e).backgroundColor);
      await primary.hover(); assert.notEqual(await primary.evaluate(e => getComputedStyle(e).backgroundColor), before);
      await page.mouse.move(0, 0); await page.locator('h1').click();
      await page.screenshot({ path: path.join(outputPath, `${name}.png`), fullPage: true });
      assert.deepEqual(errors, []); checks.push({ name, width, colorScheme, overflow: false, keyboard: true, images: true, hover: true });
      await context.close();
    }
    for (const state of ['valid', 'unsafe', 'offline', 'oversize']) {
      const context = await browser.newContext(); const page = await context.newPage();
      await page.route('**/release.json', route => {
        if (state === 'offline') return route.abort();
        const r = { version: '0.1.0', preview: true, signed: false };
        for (const kind of ['installer', 'portable']) r[kind] = `https://github.com/joongi007/dlpad/releases/download/v0.1.0/dlpad-0.1.0-windows-x86_64-${kind}.zip`;
        if (state === 'unsafe') r.portable = 'https://evil.example/file.zip';
        return route.fulfill({ contentType: 'application/json', body: state === 'oversize' ? ' '.repeat(8193) : JSON.stringify({ schema: 1, release: r }) });
      });
      await page.goto(base); await page.waitForFunction(() => !document.getElementById('release-state').textContent.includes('확인하고 있습니다'));
      const href = await page.locator('#installer-link').getAttribute('href');
      assert.equal(href, state === 'valid' ? 'https://github.com/joongi007/dlpad/releases/download/v0.1.0/dlpad-0.1.0-windows-x86_64-installer.zip' : 'https://github.com/joongi007/dlpad/releases');
      checks.push({ releaseState: state, passed: true }); await context.close();
    }
    fs.writeFileSync(path.join(outputPath, 'result.json'), JSON.stringify({ headlessOnly: true, boundHost: '127.0.0.1', checks }, null, 2));
    console.log(`${checks.length} browser scenarios PASS`);
  } finally {
    if (browser) await browser.close();
    await new Promise(resolve => server.close(resolve));
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
