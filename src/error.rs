use std::fmt::Formatter;
use std::io::Error;

use crate::convert_error;

#[derive(Eq, PartialEq)]
pub enum TcpIpError {
    DataTimeout,
    TcpTimeout,
    Other(String),
}

impl std::error::Error for TcpIpError {}

impl std::fmt::Display for TcpIpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TcpIpError::DataTimeout => write!(f, "Data Timed out"),
            TcpIpError::TcpTimeout => write!(f, "TCP Socket Timed out"),
            TcpIpError::Other(e) => write!(f, "{}", e),
        }
    }
}

impl std::fmt::Debug for TcpIpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl TcpIpError {
    pub fn new<T: AsRef<str>>(msg: T) -> Self {
        TcpIpError::Other(msg.as_ref().to_owned())
    }
}

impl From<std::io::Error> for TcpIpError {
    fn from(e: Error) -> Self {
        let e_string = e.to_string();

        if e.kind() == std::io::ErrorKind::WouldBlock
            || e.kind() == std::io::ErrorKind::TimedOut
            || e_string.contains("(os error 10053)")
        {
            TcpIpError::TcpTimeout
        } else {
            TcpIpError::new(e_string)
        }
    }
}

convert_error!(std::fmt::Error);
convert_error!(std::num::ParseIntError);
convert_error!(std::num::ParseFloatError);
convert_error!(std::net::AddrParseError);
convert_error!(std::string::FromUtf8Error);
convert_error!(std::array::TryFromSliceError);

#[macro_export]
macro_rules! convert_error {
    ($err:path) => {
        impl From<$err> for TcpIpError {
            fn from(e: $err) -> Self {
                Self::new(e.to_string())
            }
        }
    };
}
