wit_bindgen_guest_rust::generate!("databook");

struct Plugin;

impl PluginSystem for Plugin {
    fn invoke() -> String {
        let mut hello = "Hello, ".to_string();
        let req = host::HttpRequest {
            method: "get".into(),
            url: "http://google.com/",
            params: "",
            body: "",
            headers: &[host::HttpHeaderParam {
                key: "User-Agent",
                value: "databook",
            }],
        };
        host::log(host::LogLevel::Info, "Starting request");
        host::http(req);
        host::log(host::LogLevel::Info, "Finished request");
        hello.push_str("World");
        hello
    }
}
export_plugin_system!(Plugin);