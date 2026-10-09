import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, copyFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';

function fixture(t) {
  const root = mkdtempSync(join(tmpdir(), 'clanzi-release-'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  for (const dir of ['tools', 'src', 'src-tauri/src']) mkdirSync(join(root, dir), { recursive: true });
  copyFileSync(new URL('../tools/release-version.mjs', import.meta.url), join(root, 'tools/release-version.mjs'));
  const json = (file, value) => writeFileSync(join(root, file), JSON.stringify(value));
  json('src-tauri/tauri.conf.json', { version: '0.1.0' });
  json('package.json', { version: '0.1.0' });
  json('package-lock.json', { version: '0.1.0', packages: { '': { version: '0.1.0' } } });
  for (const [path, name, entry] of [['', 'clanzi-core', 'src/lib.rs'], ['src-tauri', 'clanzi', 'src/main.rs']]) {
    writeFileSync(join(root, path, 'Cargo.toml'), `[package]\nname = "${name}"\nversion = "0.1.0"\nedition = "2021"\n`);
    writeFileSync(join(root, path, entry), '');
  }
  return {
    root, json,
    run: (...args) => {
      const result = spawnSync(process.execPath, [join(root, 'tools/release-version.mjs'), ...args], { encoding: 'utf8' });
      assert.ifError(result.error);
      return result;
    },
  };
}

test('release version works from an external cwd and accepts only the matching tag', t => {
  const f = fixture(t);
  for (const args of [[], ['v0.1.0']]) {
    const result = f.run(...args);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stdout.trim(), '0.1.0');
  }
  for (const tag of ['v0.2.0', '0.1.0', 'v0.1.0-rc.1', 'v0.1.0\nanything']) {
    assert.notEqual(f.run(tag).status, 0, tag);
  }
});

test('release fails when a JavaScript manifest or lock version differs', t => {
  const f = fixture(t);
  f.json('package.json', { version: '0.2.0' });
  assert.match(f.run().stderr, /package.json version 0.2.0/);
  f.json('package.json', { version: '0.1.0' });
  f.json('package-lock.json', { version: '0.1.0', packages: { '': { version: '0.2.0' } } });
  assert.match(f.run().stderr, /root package version 0.2.0/);
});

test('release rejects Rust version drift and unsupported version formats', t => {
  const f = fixture(t);
  writeFileSync(join(f.root, 'src-tauri/Cargo.toml'), '[package]\nname="clanzi"\nversion="0.2.0"\nedition="2021"\n');
  assert.match(f.run().stderr, /src-tauri\/Cargo.toml version 0.2.0/);
  for (const version of ['01.0.0', 'v0.1.0', '0.1.0-rc.1', '0.1.0+build']) {
    f.json('src-tauri/tauri.conf.json', { version });
    const result = f.run();
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /MAJOR.MINOR.PATCH/);
  }
});
