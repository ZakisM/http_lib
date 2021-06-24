use std::io::{Read, Write};

use crate::error::TcpIpError;
use crate::header_item::HeaderItem;
use crate::header_map::HeaderMap;
use crate::Result;

#[derive(Debug, Clone)]
pub struct ResponseHeader {
    pub version: f32,
    pub status_code: u16,
    pub reason_phrase: String,
    pub headers: Option<HeaderMap>,
}

impl ResponseHeader {
    pub fn new<T: AsRef<str>>(
        version: f32,
        status_code: u16,
        reason_phrase: T,
        headers: Option<HeaderMap>,
    ) -> Self {
        ResponseHeader {
            version,
            status_code,
            reason_phrase: reason_phrase.as_ref().to_owned(),
            headers,
        }
    }
}

impl HeaderItem for ResponseHeader {
    fn headers(&self) -> &Option<HeaderMap> {
        &self.headers
    }

    fn headers_mut(&mut self) -> &mut Option<HeaderMap> {
        &mut self.headers
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut header_str = String::new();

        bytes
            .take(bytes.len() as u64)
            .read_to_string(&mut header_str)?;

        let mut header_str_lines = header_str.lines();

        let mut status_line = header_str_lines
            .next()
            .ok_or_else(|| TcpIpError::new("Failed to read HTTP Response Status line"))?
            .split_whitespace();

        let version = status_line
            .next()
            .map(|v| v.replace("HTTP/", ""))
            .ok_or_else(|| TcpIpError::new("Failed to read HTTP Response Version"))?
            .parse::<f32>()?;

        let status_code = status_line
            .next()
            .ok_or_else(|| TcpIpError::new("Failed to read HTTP Response Status code"))?
            .parse::<u16>()?;

        let reason_phrase = status_line
            .next()
            .ok_or_else(|| TcpIpError::new("Failed to read HTTP Response Reason phrase"))?
            .to_owned();

        let headers = HeaderMap::from_header_lines(&mut header_str_lines);

        Ok(ResponseHeader {
            version,
            status_code,
            reason_phrase,
            headers,
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();

        write!(
            bytes,
            "HTTP/{} {} {}\r\n",
            self.version, self.status_code, self.reason_phrase
        )?;

        self.write_headers(&mut bytes)?;

        write!(bytes, "\r\n")?;

        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::body_type::BodyType;
    use crate::header_item::HeaderItem;
    use crate::response::response_header::ResponseHeader;

    #[test]
    fn test_from_bytes_to_bytes() {
        let raw_request = String::from("HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\n");

        let header =
            ResponseHeader::from_bytes(raw_request.as_bytes()).expect("Failed to read request");

        assert_eq!(header.version, 1.1);
        assert_eq!(header.status_code, 200);
        assert_eq!(header.reason_phrase, "OK");

        let headers = header.headers.as_ref().expect("Headers was None");

        assert_eq!(headers.get("Content-Length"), Some("13"));

        assert_eq!(
            header
                .to_bytes()
                .expect("Failed to convert header to bytes"),
            raw_request.as_bytes()
        );
    }

    #[test]
    fn test_body_type_fixed() {
        let raw_request = String::from("HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\n");

        let header =
            ResponseHeader::from_bytes(raw_request.as_bytes()).expect("Failed to read request");

        assert_eq!(header.body_type(), Some(BodyType::Fixed(13)))
    }

    #[test]
    fn test_body_type_chunked() {
        let raw_request = String::from("HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n");

        let header =
            ResponseHeader::from_bytes(raw_request.as_bytes()).expect("Failed to read request");

        assert_eq!(header.body_type(), Some(BodyType::Chunked))
    }
}
