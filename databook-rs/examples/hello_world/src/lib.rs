wit_bindgen_guest_rust::generate!("databook");

struct Plugin;

impl PluginSystem for Plugin {
    fn invoke() -> String {
        let mut hello = "Hello, ".to_string();
        let req = runtime::HttpRequest {
            method: "get".into(),
            url: "http://google.com/",
            params: "",
            body: "",
            headers: &[runtime::HttpHeaderParam {
                key: "User-Agent",
                value: "databook",
            }],
        };
        runtime::log(runtime::LogLevel::Info, "Starting request");
        runtime::http(req);
        runtime::log(runtime::LogLevel::Info, "Finished request");
        hello.push_str("World");
        hello
    }
}
