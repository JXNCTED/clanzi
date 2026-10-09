.PHONY: dev demo test check build deb
dev:
	npm run tauri -- dev
demo:
	npm run tauri -- dev -- --demo
test:
	cargo test --offline
	node tests/frontend.test.mjs
check:
	cargo fmt --all -- --check
	cargo clippy --offline --all-targets -- -D warnings
	npm run build
	$(MAKE) test
build:
	npm run build
	cargo build --manifest-path src-tauri/Cargo.toml --release --features custom-protocol --locked
deb:
	sh tools/package-deb.sh
