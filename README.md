# Databook - Notebook with wasm plugins

## What?

The idea is to create a notebook system, where users can write WASM plugins in order to enrich their experience.

For example if I want to let users query prometheus while using the notebook, I would install the prometheus plugin on the databook-rs. And in the front-end I would write something like:


```
@plugin=prometheus
my_metryc[5m]
```

This would be send to the backend, the plugin would query prometheus and return the result to the front-end. The front-end would parse the result and plot it.

The cool thing is that all plugins are run as WASM module, so they are isolated from the host. It's almost like databook-rs was in reality a FaaS and the plugins the functions. If you want, you can actually use databook-rs as a (limited) FaaS, instead of having the front-end calling it, you can have your own apps using it as backend, you just need to generate the client code from the protobufs. 

## Status

WIP

## Why?

To learn a bit of wasm and rust

## How?

### databook-rs

(**current you need rust nightly to build the project**)

(**designed by [Elias](https://elias.sh)**)

Databook-rs is the brain behind the backend wasm plugins, it's a simple runtime platform that responds to the inputs from users selecting the appropriate plugin. `databook-rs/examples/plugins` contain examples of how the plugins must be written. While `plugins` contain some of those examples compiled down to wasm already. 

The plugins are loaded from a specific folder. Plugins are made of a `config.toml` and a `plugin.wasm`. The `plugin.wasm` must conform with `wit/plugin.wit` interface, you can use wit-bindgen to generate the boilerplate code. The runtime exposes to all the plugins `wit/runtime.wit` (e.g. http_request methods, env variables). The `config.toml` must specify which env variables it want access to, and only those will be given to the service (e.g. for credentials, options and so on).

All plugins are run independently of each other and from previous execution. So it's not possible to leak information between two requests.

Databook-rs uses wasmtime.

Databook-rs exposes a grpc service. That grpc server is super simple and only contains one method: `get`. The input has the name of the plugin and its input. The response is a string.

If you want to test out the databook-rs without any front-end, you can start it with a simple: `cargo run --bin server` and you can send a super simple request using: `cargo run --bin client`.

### web

(**designed by [Marcelo J](https://codeberg.org/marceloadsj1)**)

TODO explain


# Influenced by

- Fiberplane
- https://github.com/masmullin2000/wit-bindgen-example/blob/main/host/src/main.rs
- https://codeberg.org/era/malleable-checker 

# Collaborators 
- [Elias Jr](https://codeberg.org/era)
- [Marcelo Jr](https://codeberg.org/marceloadsj1)
- [Murilo Clemente](https://codeberg.org/muclemente)
