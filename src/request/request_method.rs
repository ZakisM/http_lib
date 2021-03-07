use std::fmt::Formatter;
use std::str::FromStr;

use crate::error::TcpIpError;
use crate::Result;

#[derive(Debug, Eq, PartialEq)]
pub enum RequestMethod {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Trace,
    Options,
    Connect,
    Patch,
}

impl FromStr for RequestMethod {
    type Err = TcpIpError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "GET" => Ok(RequestMethod::Get),
            "HEAD" => Ok(RequestMethod::Head),
            "POST" => Ok(RequestMethod::Post),
            "PUT" => Ok(RequestMethod::Put),
            "DELETE" => Ok(RequestMethod::Delete),
            "TRACE" => Ok(RequestMethod::Trace),
            "OPTIONS" => Ok(RequestMethod::Options),
            "CONNECT" => Ok(RequestMethod::Connect),
            "PATCH" => Ok(RequestMethod::Patch),
            _ => Err(TcpIpError::new("Unknown request header method")),
        }
    }
}

impl std::fmt::Display for RequestMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            RequestMethod::Get => "GET",
            RequestMethod::Head => "HEAD",
            RequestMethod::Post => "POST",
            RequestMethod::Put => "PUT",
            RequestMethod::Delete => "DELETE",
            RequestMethod::Trace => "TRACE",
            RequestMethod::Options => "OPTIONS",
            RequestMethod::Connect => "CONNECT",
            RequestMethod::Patch => "PATCH",
        };

        write!(f, "{}", res)
    }
}
