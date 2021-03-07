use std::io::{BufReader, Read};

use crate::error::TcpIpError;
use crate::util::slice_find_to_end;
use crate::Result;

#[derive(Debug, Eq, PartialEq)]
pub enum BodyType {
    Fixed(usize),
    Chunked,
}

impl BodyType {
    pub fn read_body<T>(&self, reader: &mut BufReader<T>) -> Result<Vec<u8>>
    where
        std::io::BufReader<T>: std::io::Read,
    {
        match *self {
            BodyType::Fixed(content_length) => {
                let mut body = Vec::with_capacity(content_length);

                match reader.take(content_length as u64).read_to_end(&mut body) {
                    Ok(_) => Ok(body),
                    Err(e) => Err(TcpIpError::from(e)),
                }
            }
            BodyType::Chunked => {
                let mut data = Vec::new();
                let mut chunk_data = Vec::new();

                loop {
                    let chunk_size_end = slice_find_to_end(chunk_data.as_slice(), &[13, 10]);

                    if let Some(chunk_size_end) = chunk_size_end {
                        let chunk_size_bytes = &chunk_data[..chunk_size_end];

                        let chunk_size = match u64::from_str_radix(
                            &String::from_utf8_lossy(chunk_size_bytes),
                            16,
                        ) {
                            Ok(c) => c,
                            Err(e) => {
                                return Err(TcpIpError::from(e));
                            }
                        };

                        // clear current chunk data which will remove the chunk size bytes
                        chunk_data.clear();

                        reader.take(chunk_size).read_to_end(&mut chunk_data)?;

                        // consume /r/n bytes
                        reader.take(2).read_to_end(&mut Vec::new())?;

                        // 0 indicates the end of the chunks
                        if chunk_size != 0 {
                            data.append(&mut chunk_data);
                        } else {
                            return Ok(data);
                        }
                    }

                    if let Err(e) = reader.take(1).read_to_end(&mut chunk_data) {
                        return Err(TcpIpError::from(e));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use crate::body_type::BodyType;

    #[test]
    fn test_read_chunked() {
        let raw_data = "7\r\nMozilla\r\n9\r\nDeveloper\r\n7\r\nNetwork\r\n0\r\n\r\n";

        let mut reader = BufReader::new(raw_data.as_bytes());
        let bytes = BodyType::Chunked
            .read_body(&mut reader)
            .expect("Failed to read body");
        assert_eq!(
            String::from_utf8(bytes).expect("Failed to convert body to String"),
            "MozillaDeveloperNetwork"
        );

        let raw_data = "4\r\nWiki\r\n6\r\npedia \r\nE\r\nin \r\n\r\nchunks.\r\n0\r\n\r\n";

        let mut reader = BufReader::new(raw_data.as_bytes());
        let bytes = BodyType::Chunked
            .read_body(&mut reader)
            .expect("Failed to read body");
        assert_eq!(
            String::from_utf8(bytes).expect("Failed to convert body to String"),
            "Wikipedia in \r\n\r\nchunks."
        );
    }
}
