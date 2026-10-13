# Repository Guidelines

## Project Structure & Module Organization

clanzi: Linux x86_64 AU05 mapper using Tauri 2/React/TypeScript/Rust. Read `README.zh.md` and `docs/configuration.zh.md` first.

- `ui/`: editor, bilingual text, IPC; `domain.ts` validates frontend configuration.
- `src/`: configuration/storage, protocol, runtime, Linux hidraw/uinput, service coordination.
- `src-tauri/`: window, tray, dialogs; separate Cargo manifest/lockfile; assets in `icons/`.
- `tests/`, `tools/`: automated checks, browser smoke test, packaging.
- `packaging/`, `docs/`, `examples/`: installation rules, documentation/screenshots, sample configuration.

Upstream specifications: `cross-platform-replication-spec*.md`; reference checkout: `../olanzi`, if present.

## Build, Test, and Development Commands

Use Node.js 24, stable Rust, and README dependencies. Preserve all three lockfiles. Prefix workspace commands with `rtk`; use `rtk proxy` for unwrapped commands.

```sh
rtk proxy make dev    # Native development
rtk proxy make demo   # Isolated native demo
rtk proxy make build  # Production frontend/native binary
rtk proxy make check  # Format, core Clippy, build, tests
rtk proxy make test   # Core, frontend, release, packaging tests
rtk proxy make deb    # Debian package
rtk proxy cargo clippy --manifest-path src-tauri/Cargo.toml --offline --all-targets -- -D warnings
rtk proxy node tools/ui-smoke.mjs # Requires built dist/ and Playwright Chromium
```

## Coding Style & Naming Conventions

Match existing style: four-space Rust, two-space TypeScript, single-quoted imports, semicolons. Use Rust `snake_case`, React/types `PascalCase`, functions `camelCase`. Use rustfmt/Clippy and strict TypeScript. State assumptions and verification before editing; preserve existing changes. Avoid unrelated refactors/dependencies/abstractions. Update `ui/i18n.ts` and user documentation in both languages.

## Testing Guidelines

Use Rust `#[test]`, Node `node:test`, Python `unittest`, and Playwright. Name tests by behavior; retain `*.test.mjs` frontend filenames. Cover affected boundaries/failures; documentation needs link/instruction checks. Report performed checks and environment limitations in PRs.

## Configuration & Runtime Safety

Edits/imports change drafts; successful saves apply mappings; exports use saved mappings. Preserve version-2 `hold` compatibility. Buttons follow physical duration without thresholds; rotation sends taps. Release shared keys only after their last owner; stopping paths must release keys, retaining failed-cleanup debt. Preserve command allowlists; exclude firmware/lighting/onboard-keymap writes and ordinary keyboard monitoring. Demos cannot access hardware or emit keys. Never log keyboard text/full mappings. Quit old instances before testing; run GUI without root.

## Commit & Pull Request Guidelines

Follow historical `feat: ...` subjects: imperative `type: description` messages. Preserve existing subjects/bodies/trailers when amending unless requested otherwise. Codex-authored commits require:

```text
Co-authored-by: Codex <noreply@openai.com>
```

Review diffs/staging and `.gitignore`; exclude artifacts, dependencies, logs, credentials, and personal configuration. PRs explain behavior, relevant issues, checks/limitations, and screenshots for UI changes.
