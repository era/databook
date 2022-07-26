wit_bindgen_rust::export!("../../../wit/plugin.wit");

struct Plugin;

impl plugin::Plugin for Plugin {
    fn invoke(input: String) -> String {
        let mut hello = "Hello, ".to_string();
        hello.push_str(&input);
        hello
    }
}
