run:
	SKIP_WASM_BUILD= cargo run -- --dev --execution=native -lruntime=debug
check:
	SKIP_WASM_BUILD= cargo check
build:
	SKIP_WASM_BUILD= cargo build
purge:
	SKIP_WASM_BUILD= cargo run -- purge-chain --dev -y
restart: purge run