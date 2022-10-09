use hyper::header::{HeaderMap, HeaderName, HeaderValue};
use crate::wasm::runtime::HttpHeaderParam;
use crate::wasm::runtime::HttpHeaderResult;

pub fn build_http_url(uri: &str, params: &str) -> String {
    format!("{}?{}", uri, params)
}

pub fn http_headers_from_runtime(
    headers: &Vec<HttpHeaderParam>,
    mut req: hyper::http::request::Builder,
) -> hyper::http::request::Builder {
    for header in headers {
        req = req.header(
            header.key.parse::<HeaderName>().unwrap(), 
            header.value.to_string().parse::<HeaderValue>().unwrap(),
        )
    }
    req
}

pub fn http_headers_to_runtime(header_map: HeaderMap) -> Vec<HttpHeaderResult> {
    let mut runtime_headers = Vec::<HttpHeaderResult>::new();
    for (key, value) in header_map {
        let runtime_header = HttpHeaderResult { 
            key: key.unwrap().as_str().into(), 
            value: value.to_str().unwrap().into() 
        };
        runtime_headers.push(runtime_header);
    }
    runtime_headers
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::http::request::Builder;

    impl PartialEq for HttpHeaderResult {
        fn eq(&self, other: &Self) -> bool {
            self.key == other.key && self.value == other.value
        }
    }

    #[test]
    fn test_build_http_url() {
        let url = build_http_url("http://www.elias.sh/", "ab=1&aa=2");
        assert_eq!(url, "http://www.elias.sh/?ab=1&aa=2");
    }

    #[test]
    fn test_http_headers_from_runtime() {
        let mut headers = Vec::<HttpHeaderParam>::new();
        headers.push(HttpHeaderParam { key: "content", value: "x" });
        headers.push(HttpHeaderParam { key: "something", value: "y" });

        let mut header_map = HeaderMap::new();
        header_map.insert(
            "content".to_string().parse::<HeaderName>().unwrap(),
            "x".to_string().parse::<HeaderValue>().unwrap(),
        );
        header_map.insert(
            "something".to_string().parse::<HeaderName>().unwrap(),
            "y".to_string().parse::<HeaderValue>().unwrap(),
        );

        assert_eq!(
            &header_map,
            http_headers_from_runtime(&headers, Builder::new())
                .headers_ref()
                .unwrap()
        )
    }

    #[test]
    fn test_http_headers_to_runtime() {
        let mut header_map = HeaderMap::new();
        header_map.insert(
            "content".to_string().parse::<HeaderName>().unwrap(),
            "x".to_string().parse::<HeaderValue>().unwrap(),
        );
        header_map.insert(
            "something".to_string().parse::<HeaderName>().unwrap(),
            "y".to_string().parse::<HeaderValue>().unwrap(),
        );

        assert_eq!([ 
            HttpHeaderResult { key: "content".to_string(), value: "x".to_string()}, 
            HttpHeaderResult { key: "something".to_string(), value: "y".to_string()}
            ].to_vec(), 
            http_headers_to_runtime(header_map))
    }
}
