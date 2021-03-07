use crate::error::TcpIpError;

pub mod body_type;
pub mod config;
pub mod error;
pub mod header_item;
pub mod header_map;
pub mod http_item;
pub mod request;
pub mod response;
pub mod stream_helper;
pub mod util;

type Result<T> = std::result::Result<T, TcpIpError>;
