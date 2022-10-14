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
        runtime::http(req);
        hello.push_str("World");
        hello
    }
}
