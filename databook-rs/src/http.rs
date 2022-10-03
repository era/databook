use hyper::header::{HeaderName, HeaderValue};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_http_url() {
        let url = build_http_url("http://www.elias.sh/", "ab=1&aa=2");
        assert_eq!(url, "http://www.elias.sh/?ab=1&aa=2");
    }
}
