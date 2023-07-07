wit_bindgen::generate!("plugin-system");

use databook::plugin::host;

struct Plugin;

impl PluginSystem for Plugin {
    fn invoke() -> String {
        let mut hello = "Hello, ".to_string();
        let req = host::HttpRequest {
            method: "get".into(),
            url: "http://google.com/".into(),
            params: "".into(),
            body: "".into(),
            headers: (&[host::HttpHeader {
                key: "User-Agent".into(),
                value: "databook".into(),
            }])
                .to_vec(),
        };
        host::log(host::LogLevel::Info, "Starting request");
        let _ = host::http(&req);
        host::log(host::LogLevel::Info, "Finished request");
        hello.push_str("World");
        hello
    }
}
export_plugin_system!(Plugin);
