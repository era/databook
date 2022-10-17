wit_bindgen_guest_rust::export!("../../../wit/plugin.wit");
wit_bindgen_guest_rust::import!("../../../wit/runtime.wit");

struct Plugin;

impl plugin::Plugin for Plugin {
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
        runtime::log("Starting request", runtime::LogLevel::Info);
        runtime::http(req);
        runtime::log("Finished request", runtime::LogLevel::Info);
        hello.push_str("World");
        hello
    }
}
