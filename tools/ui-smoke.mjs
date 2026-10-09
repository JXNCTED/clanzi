import { chromium } from 'playwright';
import { readFile, mkdir } from 'node:fs/promises';
import { resolve, extname } from 'node:path';
import assert from 'node:assert/strict';

const browser = await chromium.launch({ headless: true, executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE, args: ['--no-sandbox'] });
const errors = [];
await mkdir('artifacts', { recursive: true });
const context = await browser.newContext({ viewport: { width: 1024, height: 768 }, deviceScaleFactor: 1, locale: 'en-US' });
const page = await context.newPage();
page.on('pageerror', e => errors.push(e.message));
await page.route('**/*', async route => {
  const url = new URL(route.request().url());
  if (url.hostname !== 'clanzi.test') { await route.abort(); return; }
  const file = resolve('dist', url.pathname === '/' ? 'index.html' : `.${url.pathname}`);
  if (!file.startsWith(`${resolve('dist')}/`)) { await route.abort(); return; }
  const type = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.png': 'image/png' }[extname(file)] ?? 'application/octet-stream';
  try { await route.fulfill({ body: await readFile(file), contentType: type }); } catch { await route.fulfill({ status: 404, body: 'Not found' }); }
});
async function waitState(predicate) {
  await page.waitForFunction(predicate, undefined, { timeout: 5000 });
}
try {
  await page.goto('http://clanzi.test');
  await page.getByRole('heading', { name: 'Make every control your own.' }).waitFor();
  const save = page.getByRole('button', { name: 'Save to computer', exact: true });
  assert(await save.isEnabled());
  assert.equal(await page.getByRole('group', { name: 'Button behavior' }).count(), 0);
  assert(await page.getByRole('button', { name: 'Undo changes' }).isDisabled());
  assert(await page.getByRole('button', { name: 'Ctrl', exact: true }).isDisabled());
  await page.getByRole('button', { name: 'Letters', exact: true }).click();
  await page.getByRole('button', { name: 'C', exact: true }).click();
  await page.getByRole('button', { name: 'Ctrl', exact: true }).click();
  await waitState(() => document.querySelector('.combination')?.textContent?.includes('Ctrl+C'));
  await page.getByRole('button', { name: 'B', exact: true }).click();
  await waitState(() => document.querySelector('.combination')?.textContent?.includes('Ctrl+B'));
  await page.getByRole('button', { name: 'C', exact: true }).click();
  await page.getByRole('textbox', { name: 'Search keys…' }).fill('no-such-key');
  await page.getByText('No matching keys', { exact: true }).waitFor();
  assert((await page.locator('.combination').textContent()).includes('Ctrl+C'));
  await page.getByRole('textbox', { name: 'Search keys…' }).fill('');
  await save.click();
  await waitState(() => document.querySelector('.save-state')?.textContent?.includes('Saved to this computer'));
  assert(await save.isDisabled());
  await page.screenshot({ path: 'artifacts/keymap-en.png', fullPage: true });
  await page.getByRole('button', { name: 'Rotate right: No action', exact: true }).click();
  assert.equal(await page.getByRole('group', { name: 'Button behavior' }).count(), 0);
  await page.getByRole('button', { name: 'Top key: Ctrl + C', exact: true }).click();
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await page.getByRole('button', { name: '简体中文', exact: true }).click();
  await page.getByRole('button', { name: '键位', exact: true }).waitFor();
  await page.getByRole('button', { name: '键位', exact: true }).click();
  assert((await page.locator('.combination').textContent()).includes('Ctrl+C'));
  await page.screenshot({ path: 'artifacts/keymap-zh.png', fullPage: true });
  await page.getByRole('button', { name: '不执行动作', exact: true }).click();
  assert(await page.getByRole('button', { name: 'Ctrl', exact: true }).isDisabled());
  await page.getByRole('button', { name: 'A A', exact: true }).click();
  assert.equal(await page.getByRole('button', { name: 'Ctrl', exact: true }).getAttribute('aria-pressed'), 'false');
  await page.getByRole('button', { name: '设置', exact: true }).click();
  await page.getByRole('button', { name: 'English', exact: true }).click();
  await page.getByRole('button', { name: 'Export saved configuration', exact: true }).waitFor();
  const downloadEvent = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Export saved configuration', exact: true }).click();
  const download = await downloadEvent;
  const exported = JSON.parse(await readFile(await download.path(), 'utf8'));
  assert.equal(exported.version, 1);
  assert.deepEqual(exported.bindings.key_top, { kind: 'key', usage: 6, modifiers: ['ctrl'] });
  const chooserEvent = page.waitForEvent('filechooser');
  await page.getByRole('button', { name: 'Import configuration', exact: true }).click();
  const chooser = await chooserEvent;
  await chooser.setFiles('examples/keymap.json');
  await page.getByRole('dialog').waitFor();
  await page.getByRole('button', { name: 'Cancel', exact: true }).click();
  await page.getByRole('button', { name: 'Quit Olanzi', exact: true }).click();
  await page.getByRole('heading', { name: 'Save your changes before leaving?' }).waitFor();
  await page.getByRole('button', { name: 'Cancel', exact: true }).click();
  await page.screenshot({ path: 'artifacts/settings-en.png', fullPage: true });
  await page.getByRole('button', { name: 'Keymap', exact: true }).click();
  await page.setViewportSize({ width: 512, height: 384 });
  assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
  await page.getByRole('button', { name: 'Save to computer', exact: true }).scrollIntoViewIfNeeded();
  const bounds = await page.getByRole('button', { name: 'Save to computer', exact: true }).boundingBox();
  assert(bounds && bounds.y >= 0 && bounds.y + bounds.height <= 384);
  await page.screenshot({ path: 'artifacts/keymap-compact.png', fullPage: true });
  // Tab 导航必须到达可操作元素；不依赖鼠标才能保存。
  await page.keyboard.press('Tab');
  assert(await page.evaluate(() => ['BUTTON', 'INPUT', 'A', 'SUMMARY'].includes(document.activeElement.tagName)));
  assert.deepEqual(errors, []);
  console.log('UI smoke passed: draft, modifiers, no mode selection, save, bilingual, export snapshot, import cancellation, quit cancellation, compact layout, keyboard focus.');
} finally { await browser.close(); }
