wit_bindgen_rust::export!("../../../wit/plugin.wit");
wit_bindgen_rust::import!("../../../wit/runtime.wit");

struct Plugin;

impl plugin::Plugin for Plugin {
    fn invoke(input: String) -> String {
        let mut hello = "Hello, ".to_string();
        let req = runtime::HttpRequest {
            method: "get".into(),
            url: "http://google.com/",
            params: "",
            body: "",
            headers: "",
        };
        runtime::http(req);
        hello.push_str(&input);
        hello
    }
}
