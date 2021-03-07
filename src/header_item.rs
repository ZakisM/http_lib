use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::ops::Deref;

use crate::body_type::BodyType;
use crate::error::TcpIpError;
use crate::header_map::HeaderMap;
use crate::util::slice_find_to_end;
use crate::Result;

// List of hop by hop headers that must be
// handled by us and stripped before forwarding request
const HOP_BY_HOP_HEADERS: [&str; 11] = [
    "connection",
    "cache-control",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "proxy-connection",
    "referer",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

pub trait HeaderItem {
    fn headers(&self) -> &Option<HeaderMap>;

    fn headers_mut(&mut self) -> &mut Option<HeaderMap>;

    fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized;

    fn to_bytes(&self) -> Result<Vec<u8>>;

    fn from_reader(reader: &mut BufReader<&TcpStream>) -> Result<Self>
    where
        Self: Sized,
    {
        let data = reader.fill_buf()?;

        let headers_end = slice_find_to_end(data, &[13, 10, 13, 10]);

        if let Some(headers_end) = headers_end {
            let headers = Self::from_bytes(&data[..headers_end])?;

            reader.consume(headers_end + 4);

            Ok(headers)
        } else {
            Err(TcpIpError::DataTimeout)
        }
    }

    fn as_string(&self) -> Result<String> {
        let bytes = self.to_bytes()?;

        Ok(String::from_utf8(bytes)?)
    }

    fn body_type(&self) -> Option<BodyType> {
        if let Some(headers) = self.headers() {
            if let Some(content_length) = headers
                .get("Content-Length")
                .and_then(|cl| cl.parse::<usize>().ok())
            {
                return Some(BodyType::Fixed(content_length));
            } else if let Some(true) = headers
                .get("Transfer-Encoding")
                .map(|te| te.to_lowercase() == "chunked")
            {
                return Some(BodyType::Chunked);
            }
        }

        None
    }

    fn strip_hop_by_hop(&mut self) {
        if let Some(headers) = self.headers_mut() {
            HOP_BY_HOP_HEADERS.iter().for_each(|h| {
                headers.remove(h);
            });
        }
    }

    fn write_headers<T: Write>(&self, f: &mut T) -> Result<()> {
        if let Some(headers) = self.headers() {
            for (k, v) in headers.deref() {
                write!(f, "{}: {}\r\n", k, v)?;
            }
        }

        Ok(())
    }
}
