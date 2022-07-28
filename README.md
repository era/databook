# Databook - Notebook with wasm plugins

## What?

The base idea is to create a notebook system, where users can write WASM plugins in order to enrich their experience. Those plugins can run on the front-end or in the backend.

## Status

WIP

## Why?

To learn a bit of wasm and rust

## How?

### databook-rs

Databook-rs is the brain behind the backend wasm plugins, it's a simple runtime platform that responds to the inputs from users selecting the appropriate plugin. `databook-rs/examples/plugins` contain examples of how the plugins must be written. While `plugins` contain some of those examples compiled down to wasm already. 

Databook-rs exposes a grpc service. The plugins are loaded from a specific folder. Plugins are made of a `config.toml` and a `plugin.wasm`. The `plugin.wasm` must conform with `wit/plugin.wit` interface, you can use wit-bindgen to generate the boilerplate code. The runtime exposes to all the plugins `wit/runtime.wit` (e.g. http_request methods, env variables). The `config.toml` must specify which env variables it want access to, and only those will be given to the service (e.g. for credentials, options and so on).

All plugins are run independently of which other and from previous execution. So it's not possible to leak information between two requests.

Databook-rs uses wasmtime.

### web

TODO explain


# Influenced by

- Fiberplane
- https://github.com/masmullin2000/wit-bindgen-example/blob/main/host/src/main.rs
- https://codeberg.org/era/malleable-checker 
