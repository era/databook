# Databook - Notebook with wasm plugins

## What?

The idea is to create a SRE notebook system (something similar as Jupyter but focused on oncall), where users can write WASM plugins in order to enrich their experience.

Imagine that you are oncall for AWS, you receive a page in the middle of the night, a bit incident is happening. If you use something as Jira to update the company about the steps you are taking to solve the issue, they have to wait you run several commands, check several things, and than update the Jira. This is not great, there is very few paper trail. If someone is helping you, if you miss to sync exactly what each one is doing, you are doomed to repeat the same steps. Worst than that, in the future, if a similar situation happens again, it's most like that people won't be able to learn from the few information you added to the Jira.

Now imagine that instead of using a bunch of different software and attaching logs and screenshots to the Jira you could just open a web page, type some commands, see the output and record it forever. Also, everybody interested in the incident could just follow up what you are doing by opening the same page.

That is the idea of this software. Given that users can write different plugins, it does not matter what is the monitoring stack of your company, they all can use it. Also, you could add a few commands to: reboot machines, restart services and so on.

For example if I want to let users query prometheus while using the notebook, I would install the prometheus plugin on the databook-rs. And in the front-end I would write something like:


```
@plugin=prometheus
my_metric[5m]
```

This would be send to the backend, the plugin would query prometheus and return the result to the front-end. The front-end would parse the result and plot it.

The cool thing is that all plugins are run as WASM module, so they are isolated from the host. It's almost like databook-rs was in reality a FaaS and the plugins the functions. If you want, you can actually use databook-rs as a (limited) FaaS, instead of having the front-end calling it, you can have your own apps using it as backend, you just need to generate the client code from the protobufs. The protobufs are docummented and are at the `proto` folder.

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
