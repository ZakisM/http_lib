use std::io::{Read, Write};

use crate::error::TcpIpError;
use crate::header_item::HeaderItem;
use crate::header_map::HeaderMap;
use crate::request::request_method::RequestMethod;
use crate::Result;

#[derive(Debug, Clone)]
pub struct RequestHeader {
    pub method: RequestMethod,
    pub uri: String,
    pub version: f32,
    pub headers: Option<HeaderMap>,
}

impl RequestHeader {
    pub fn new<T: AsRef<str>>(
        method: RequestMethod,
        uri: T,
        version: f32,
        headers: Option<HeaderMap>,
    ) -> Self {
        RequestHeader {
            method,
            uri: uri.as_ref().to_owned(),
            version,
            headers,
        }
    }
}

impl HeaderItem for RequestHeader {
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

        let mut request_line = header_str_lines
            .next()
            .ok_or_else(|| TcpIpError::new("Failed to read HTTP Request line"))?
            .split_whitespace();

        let method = request_line
            .next()
            .ok_or_else(|| TcpIpError::new("Failed to read HTTP Request Method"))?
            .parse::<RequestMethod>()?;

        let uri = request_line
            .next()
            .ok_or_else(|| TcpIpError::new("Failed to read HTTP Request URI"))?;

        let version = request_line
            .next()
            .map(|v| v.replace("HTTP/", ""))
            .ok_or_else(|| TcpIpError::new("Failed to read HTTP Request Version"))?
            .parse::<f32>()?;

        let headers = HeaderMap::from_header_lines(&mut header_str_lines);

        Ok(RequestHeader {
            method,
            uri: uri.to_owned(),
            version,
            headers,
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();

        write!(
            bytes,
            "{} {} HTTP/{}\r\n",
            self.method, self.uri, self.version
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
    use crate::request::request_header::RequestHeader;
    use crate::request::request_method::RequestMethod;

    #[test]
    fn test_from_bytes_to_bytes() {
        let raw_request = String::from("GET /v1/api/episode/watch/random HTTP/2\r\nHost: pkaindex.com\r\nuser-agent: insomnia/2020.5.2\r\ncookie: __cfduid=d863d271a2081db4604f4f9ba9a131f221608908853\r\naccept: */*\r\n\r\n");

        let header =
            RequestHeader::from_bytes(raw_request.as_bytes()).expect("Failed to read request");

        assert_eq!(header.method, RequestMethod::Get);
        assert_eq!(header.uri, "/v1/api/episode/watch/random");
        assert_eq!(header.version, 2.0);

        let headers = header.headers.as_ref().expect("Headers was None");

        assert_eq!(headers.get("Host"), Some("pkaindex.com"));
        assert_eq!(headers.get("user-agent"), Some("insomnia/2020.5.2"));
        assert_eq!(
            headers.get("cookie"),
            Some("__cfduid=d863d271a2081db4604f4f9ba9a131f221608908853")
        );
        assert_eq!(headers.get("accept"), Some("*/*"));

        assert_eq!(
            header
                .to_bytes()
                .expect("Failed to convert header to bytes"),
            raw_request.as_bytes()
        );
    }

    #[test]
    fn test_body_type_fixed() {
        let raw_request = String::from("GET /v1/api/episode/watch/random HTTP/1.1\r\nHost: localhost:5678\r\nUser-Agent: insomnia/2020.5.2\r\nContent-Type: application/json\r\nAccept: */*\r\nContent-Length: 20\r\n\r\n");
        let header =
            RequestHeader::from_bytes(raw_request.as_bytes()).expect("Failed to read request");

        assert_eq!(header.body_type(), Some(BodyType::Fixed(20)))
    }

    #[test]
    fn test_body_type_chunked() {
        let raw_request = String::from("GET /v1/api/episode/watch/random HTTP/1.1\r\nHost: localhost:5678\r\nUser-Agent: insomnia/2020.5.2\r\nContent-Type: application/json\r\nAccept: */*\r\nTransfer-Encoding: Chunked\r\n\r\n");
        let header =
            RequestHeader::from_bytes(raw_request.as_bytes()).expect("Failed to read request");

        assert_eq!(header.body_type(), Some(BodyType::Chunked))
    }

    #[test]
    fn test_strip_hop_by_hop() {
        let raw_request = String::from("GET /v1/api/episode/watch/random HTTP/1.1\r\nHost: localhost:5678\r\nUser-Agent: insomnia/2020.5.2\r\nContent-Type: application/json\r\nAccept: */*\r\nTransfer-Encoding: Chunked\r\n\r\n");
        let mut header =
            RequestHeader::from_bytes(raw_request.as_bytes()).expect("Failed to read request");

        assert!(header
            .headers
            .as_ref()
            .expect("Headers was None")
            .get("transfer-encoding")
            .is_some());

        header.strip_hop_by_hop();

        assert!(header
            .headers
            .as_ref()
            .expect("Headers was None")
            .get("transfer-encoding")
            .is_none());
    }
}
