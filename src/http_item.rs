use std::fmt::{Display, Formatter};
use std::io::BufReader;
use std::net::TcpStream;

use crate::header_item::HeaderItem;
use crate::Result;

pub trait HttpItem {
    type HeaderType: HeaderItem;

    fn item_name(&self) -> &str;

    fn header(&self) -> &Self::HeaderType;

    fn body(&self) -> Option<Vec<u8>>;

    fn new(header: Self::HeaderType, body: Option<Vec<u8>>) -> Self
    where
        Self: Sized;

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = self.header().to_bytes()?;

        if let Some(mut body) = self.body() {
            bytes.append(&mut body);
        }

        Ok(bytes)
    }

    fn as_string(&self) -> Result<String> {
        let bytes = self.to_bytes()?;

        Ok(String::from_utf8(bytes)?)
    }

    fn from_reader(reader: &mut BufReader<&TcpStream>) -> Result<Self>
    where
        Self: Sized,
    {
        let header = Self::HeaderType::from_reader(reader)?;

        let body = if let Some(b) = header.body_type() {
            Some(b.read_body(reader)?)
        } else {
            None
        };

        Ok(Self::new(header, body))
    }

    fn display(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.header().as_string() {
            Ok(header) => {
                write!(f, "{}", header)?;

                if let Some(body) = self.body() {
                    match String::from_utf8(body) {
                        Ok(body) => write!(f, "{}", body)?,
                        Err(_) => write!(f, "Binary data")?,
                    };

                    write!(f, "\n\n")?;
                }

                Ok(())
            }
            Err(e) => {
                write!(f, "Failed to display {} - {}", self.item_name(), e)
            }
        }
    }

    fn pretty_print(&self, proxy_server_name: &str)
    where
        Self: Display,
    {
        let line_length = 60;

        println!("{}", "-".repeat(line_length));
        println!("{} [{}]", self.item_name(), proxy_server_name);
        println!("{}\n", "-".repeat(line_length));
        print!("{}", self);
    }
}
