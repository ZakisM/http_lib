use std::convert::TryFrom;
use std::fmt::Formatter;

use crate::error::ErrorExt;
use crate::header_map::HeaderMap;
use crate::http_item::HttpItem;
use crate::response::response_header::ResponseHeader;
use crate::response::response_status::ResponseStatus;
use crate::Result;

pub mod response_header;
pub mod response_status;

#[derive(Debug, Default)]
pub struct ResponseBuilder {
    version: f32,
    status_code: Option<u16>,
    reason_phrase: Option<String>,
    headers: Option<HeaderMap>,
    body: Option<Vec<u8>>,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        ResponseBuilder {
            version: 1.1,
            ..Self::default()
        }
    }

    pub fn version(mut self, version: f32) -> Self {
        self.version = version;
        self
    }

    pub fn status_code(mut self, status_code: u16) -> Self {
        self.status_code = Some(status_code);
        self
    }

    pub fn reason_phrase<U: AsRef<str>>(mut self, reason_phrase: U) -> Self {
        self.reason_phrase = Some(reason_phrase.as_ref().to_owned());
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

    pub fn body(self, body: Vec<u8>) -> Self {
        let mut new = self.header("Content-Length", &body.len().to_string());
        new.body = Some(body);
        new
    }

    pub fn build(self) -> Result<Response> {
        let version = self.version;
        let status_code = self.status_code.context("Missing status_code")?;

        let reason_phrase = if let Some(reason_phrase) = self.reason_phrase {
            reason_phrase
        } else {
            ResponseStatus::try_from(status_code)?.to_string()
        };

        let headers = self.headers;
        let body = self.body;

        let header = ResponseHeader::new(version, status_code, reason_phrase, headers);

        Ok(Response { header, body })
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub header: ResponseHeader,
    pub body: Option<Vec<u8>>,
}

impl HttpItem for Response {
    type HeaderType = ResponseHeader;

    fn item_name(&self) -> &str {
        "Response"
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

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.display(f)
    }
}

#[cfg(test)]
mod tests {
    use crate::header_map::HeaderMap;
    use crate::http_item::HttpItem;
    use crate::response::response_header::ResponseHeader;
    use crate::response::Response;

    #[test]
    fn test_to_bytes() {
        let request = Response {
            header: ResponseHeader {
                version: 1.1,
                status_code: 200,
                reason_phrase: "OK".to_string(),
                headers: Some(HeaderMap {
                    headers: vec![("Content-Length".to_owned(), "24".to_owned())],
                }),
            },
            body: Some(vec![
                123, 10, 9, 34, 100, 97, 116, 97, 34, 58, 32, 34, 104, 101, 108, 108, 111, 32, 122,
                97, 107, 34, 10, 125,
            ]),
        };

        let request_str_raw =
            "HTTP/1.1 200 OK\r\nContent-Length: 24\r\n\r\n{\n\t\"data\": \"hello zak\"\n}";

        let request_str = request
            .as_string()
            .expect("Failed to convert request to str");

        assert_eq!(request_str_raw, request_str);
    }
}
