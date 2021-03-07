use std::io::{BufReader, BufWriter, Write};
use std::net::{SocketAddr, SocketAddrV4, TcpStream};
use std::time::Duration;

use crate::header_item::HeaderItem;
use crate::http_item::HttpItem;
use crate::request::Request;
use crate::response::Response;
use crate::Result;

pub fn connect_remote(address: &SocketAddrV4, timeout_seconds: u64) -> Result<TcpStream> {
    let address = SocketAddr::from(*address);

    let remote_server = TcpStream::connect_timeout(&address, Duration::from_secs(timeout_seconds))?;

    setup_stream(&remote_server, timeout_seconds)?;

    Ok(remote_server)
}

pub fn setup_stream(stream: &TcpStream, timeout_seconds: u64) -> Result<()> {
    stream.set_nodelay(true)?;
    stream.set_read_timeout(Some(Duration::from_secs(timeout_seconds)))?;
    stream.set_write_timeout(Some(Duration::from_secs(timeout_seconds)))?;

    Ok(())
}

pub fn forward_request(
    proxy_server_name: &str,
    request: &mut Request,
    local_writer: &mut BufWriter<&TcpStream>,
    remote_address: &SocketAddrV4,
    timeout_seconds: u64,
) -> Result<()> {
    let remote_server = connect_remote(remote_address, timeout_seconds)?;

    let mut remote_reader = BufReader::new(&remote_server);
    let mut remote_writer = BufWriter::new(&remote_server);

    request.header.strip_hop_by_hop();

    remote_writer.write_all(&request.to_bytes()?)?;
    remote_writer.flush()?;

    let mut response = Response::from_reader(&mut remote_reader)?;

    response.header.strip_hop_by_hop();

    let body = response.body();

    if let Some(response_headers) = response.header.headers_mut() {
        if let Some(body) = body {
            response_headers.insert("Content-Length", &body.len().to_string());
        }
    }

    local_writer.write_all(&response.to_bytes()?)?;
    local_writer.flush()?;

    request.pretty_print(proxy_server_name);
    response.pretty_print(proxy_server_name);

    Ok(())
}
