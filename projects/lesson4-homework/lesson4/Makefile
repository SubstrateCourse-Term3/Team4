run:
	SKIP_WASM_BUILD= cargo run -- --dev --execution=native -lruntime=debug

toolchain:
	./scripts/init.sh

build-wasm:
	WASM_BUILD_TYPE=release cargo build

check:
	SKIP_WASM_BUILD= cargo check

check-debug:
	RUSTFLAGS="-Z external-macro-backtrace" BUILD_DUMMY_WASM_BINARY= cargo +nightly check

check-dummy:
	BUILD_DUMMY_WASM_BINARY= cargo check

build:
	SKIP_WASM_BUILD= cargo build

purge:
	SKIP_WASM_BUILD= cargo run -- purge-chain --dev -y

restart: purge run

target/debug/substrate-kitties: build

init: toolchain submodule build-wasm

submodule:
	git submodule update --init --recursive
