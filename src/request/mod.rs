use std::fmt::Formatter;

use crate::header_map::HeaderMap;
use crate::http_item::HttpItem;
use crate::request::request_header::RequestHeader;
use crate::request::request_method::RequestMethod;

pub mod request_header;
pub mod request_method;

#[derive(Debug, Default)]
pub struct RequestBuilder {
    method: Option<RequestMethod>,
    uri: Option<String>,
    version: f32,
    headers: Option<HeaderMap>,
    body: Option<Vec<u8>>,
}

impl RequestBuilder {
    pub fn new() -> Self {
        RequestBuilder {
            version: 1.1,
            ..Self::default()
        }
    }

    pub fn method(mut self, method: RequestMethod) -> Self {
        self.method = Some(method);
        self
    }

    pub fn uri<U: AsRef<str>>(mut self, uri: U) -> Self {
        self.uri = Some(uri.as_ref().to_owned());
        self
    }

    pub fn version(mut self, version: f32) -> Self {
        self.version = version;
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        if let Some(headers) = &mut self.headers {
            headers.insert(key, value)
        } else {
            let mut headers = HeaderMap::new();
            headers.insert(key, value);
            self.headers = Some(headers);
        };

        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    pub fn build(self) -> Option<Request> {
        let method = self.method?;
        let uri = self.uri?;
        let version = self.version;
        let headers = self.headers;
        let body = self.body;

        let header = RequestHeader::new(method, uri, version, headers);

        Some(Request { header, body })
    }
}

#[derive(Debug)]
pub struct Request {
    pub header: RequestHeader,
    pub body: Option<Vec<u8>>,
}

impl HttpItem for Request {
    type HeaderType = RequestHeader;

    fn item_name(&self) -> &str {
        "Request"
    }

    fn header(&self) -> &Self::HeaderType {
        &self.header
    }

    fn body(&self) -> Option<Vec<u8>> {
        self.body.clone()
    }

    fn new(header: Self::HeaderType, body: Option<Vec<u8>>) -> Self {
        Self { header, body }
    }
}

impl std::fmt::Display for Request {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.display(f)
    }
}

#[cfg(test)]
mod tests {
    use crate::header_map::HeaderMap;
    use crate::http_item::HttpItem;
    use crate::request::request_header::RequestHeader;
    use crate::request::request_method::RequestMethod;
    use crate::request::Request;

    #[test]
    fn test_to_bytes() {
        let request = Request {
            header: RequestHeader {
                method: RequestMethod::Get,
                uri: "/abc/123".to_string(),
                version: 1.1,
                headers: Some(HeaderMap {
                    headers: vec![
                        ("Content-Type".to_owned(), "application/json".to_owned()),
                        ("Accept".to_owned(), "*/*".to_owned()),
                        ("Content-Length".to_owned(), "26".to_owned()),
                    ],
                }),
            },
            body: Some(vec![
                123, 10, 9, 34, 100, 97, 116, 97, 34, 58, 32, 34, 104, 101, 108, 108, 111, 32, 119,
                111, 114, 108, 100, 34, 10, 125,
            ]),
        };

        let request_str_raw = "GET /abc/123 HTTP/1.1\r\nContent-Type: application/json\r\nAccept: */*\r\nContent-Length: 26\r\n\r\n{\n\t\"data\": \"hello world\"\n}";

        let request_str = request
            .as_string()
            .expect("Failed to convert request to str");

        assert_eq!(request_str_raw, request_str);
    }
}
