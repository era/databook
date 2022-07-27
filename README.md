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

### web

TODO explain


# Influenced by

- Fiberplane
- https://github.com/masmullin2000/wit-bindgen-example/blob/main/host/src/main.rs
- https://codeberg.org/era/malleable-checker 
