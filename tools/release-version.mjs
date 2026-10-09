import { readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const read = path => JSON.parse(readFileSync(resolve(root, path), 'utf8'));
try {
  const version = read('src-tauri/tauri.conf.json').version;
  if (!/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(version)) {
    throw Error('Release version must be MAJOR.MINOR.PATCH, without a prefix or prerelease suffix.');
  }
  const tag = process.argv[2];
  if (tag !== undefined && tag !== `v${version}`) {
    throw Error(`Tag must be v${version}; received ${JSON.stringify(tag)}.`);
  }
  const lock = read('package-lock.json');
  const versions = {
    'package.json': read('package.json').version,
    'package-lock.json': lock.version,
    'package-lock.json root package': lock.packages[''].version,
  };
  for (const manifest of ['Cargo.toml', 'src-tauri/Cargo.toml']) {
    const metadata = JSON.parse(execFileSync('cargo', [
      'metadata', '--manifest-path', manifest, '--format-version', '1', '--no-deps', '--offline', '--locked',
    ], { cwd: root, encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }));
    versions[manifest] = metadata.packages.find(p => resolve(p.manifest_path) === resolve(root, manifest)).version;
  }
  for (const [file, actual] of Object.entries(versions)) {
    if (actual !== version) throw Error(`${file} version ${actual} does not match Tauri version ${version}.`);
  }
  console.log(version);
} catch (error) {
  console.error(error.message);
  process.exitCode = 1;
}
