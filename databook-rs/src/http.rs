use hyper::header::{HeaderMap, HeaderName, HeaderValue};

pub fn build_http_url(uri: &str, params: &str) -> String {
    format!("{}?{}", uri, params)
}

pub fn http_headers_from_str(
    headers: &str,
    mut req: hyper::http::request::Builder,
) -> hyper::http::request::Builder {
    let splitted: Vec<&str> = headers.split('&').collect();

    for header in splitted {
        let header: Vec<&str> = header.split('=').collect();

        match (header.get(0), header.get(1)) {
            (Some(key), Some(value)) => {
                req = req.header(
                    key.to_string().parse::<HeaderName>().unwrap(),
                    value.to_string().parse::<HeaderValue>().unwrap(),
                )
            }
            _ => continue,
        };
    }
    return req;
}

pub fn http_headers_to_str(header_map: HeaderMap) -> String {
    let mut header_as_str = Vec::<String>::new();
    let mut headers = Vec::<String>::new();
    for (key, value) in header_map {
        header_as_str.push(key.unwrap().as_str().into()); //TODO
        header_as_str.push(value.to_str().unwrap().into()); //TODO
        headers.push(header_as_str.join("="));
        header_as_str.clear();
    }
    headers.join("&")
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::http::request::Builder;

    #[test]
    fn test_build_http_url() {
        let url = build_http_url("http://www.elias.sh/", "ab=1&aa=2");
        assert_eq!(url, "http://www.elias.sh/?ab=1&aa=2");
    }

    #[test]
    fn test_http_headers_from_str() {
        let header = "content=x&something=y";
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
            http_headers_from_str(header, Builder::new())
                .headers_ref()
                .unwrap()
        )
    }

    #[test]
    fn test_http_headers_to_str() {
        let mut header_map = HeaderMap::new();
        header_map.insert(
            "content".to_string().parse::<HeaderName>().unwrap(),
            "x".to_string().parse::<HeaderValue>().unwrap(),
        );
        header_map.insert(
            "something".to_string().parse::<HeaderName>().unwrap(),
            "y".to_string().parse::<HeaderValue>().unwrap(),
        );

        assert_eq!("content=x&something=y", http_headers_to_str(header_map))
    }
}
