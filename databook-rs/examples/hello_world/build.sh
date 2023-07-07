cargo build --target wasm32-wasi
#wasm-tools component new ./target/wasm32-wasi/debug/hello_world.wasm -o my-component.wasm --adapt ./wasi_snapshot_preview1.wasm
# downloaded from https://github.com/bytecodealliance/wasmtime/releases/tag/dev
wasm-tools component new ./target/wasm32-wasi/debug/hello_world.wasm -o my-component.wasm --adapt wasi_snapshot_preview1=./wasi_snapshot_preview1.reactor.wasm
mv my-component.wasm ../../plugins/hello_world/plugin.wasm