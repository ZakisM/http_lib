use std::fs;
use std::io::{BufReader, BufWriter, Write};
use std::net::{SocketAddrV4, TcpListener};
use std::str::FromStr;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crate::error::TcpIpError;
use crate::http_item::HttpItem;
use crate::request::Request;
use crate::stream_helper::{forward_request, setup_stream};
use crate::Result;

const CONFIG_FILE_NAME: &str = "tcp_ip_monitor_config.txt";

pub struct Config {
    pub servers: Vec<Server>,
}

impl Config {
    pub fn load() -> Result<Self> {
        if let Ok(contents) = fs::read_to_string(CONFIG_FILE_NAME) {
            let servers = contents
                .lines()
                .filter(|l| !l.starts_with('#'))
                .map(|s| Server::from_str(s))
                .collect::<std::result::Result<Vec<_>, _>>()?;

            if servers.is_empty() {
                Err(TcpIpError::new(format!(
                    "'{}' must not be empty",
                    CONFIG_FILE_NAME
                )))
            } else {
                Ok(Self { servers })
            }
        } else {
            let current_dir = std::env::current_dir()?;

            let mut f = fs::File::create(CONFIG_FILE_NAME).map_err(|e| {
                TcpIpError::new(format!("Failed to create '{}' - {}", CONFIG_FILE_NAME, e))
            })?;

            f.write_all(b"# Format [local port to listen on] [remote address to forward to] [timeout in seconds (optional - will default to 4)]\n# Example:\n# 1234 127.0.0.1:5678")
                .map_err(|e| TcpIpError::new(format!("Failed to write to '{}' - {}", CONFIG_FILE_NAME, e)))?;

            Err(TcpIpError::new(format!("Missing config file named '{}'. One has been created at '{}'. Please modify it and then restart the tcp_ip_monitor.", CONFIG_FILE_NAME, current_dir.display())))
        }
    }
}

pub struct Server {
    pub listen_port: u16,
    pub remote_address: SocketAddrV4,
    pub timeout: u64,
    pub name: String,
}

impl Server {
    pub fn start(self) -> Result<JoinHandle<()>> {
        let local_address = SocketAddrV4::from_str(&format!("127.0.0.1:{}", self.listen_port))?;
        let remote_address = self.remote_address;
        let timeout = self.timeout;
        let name = Arc::new(self.name);

        println!(
            "Proxy service started at 'http://{}'. Forwarding requests to 'http://{}'. Timeout is {} seconds.\n",
            local_address, remote_address, timeout
        );

        let local_server = TcpListener::bind(local_address)?;

        Ok(thread::spawn(move || {
            for stream in local_server.incoming() {
                match stream {
                    Ok(stream) => {
                        let name = name.clone();

                        thread::spawn(move || {
                            if let Err(e) = setup_stream(&stream, timeout) {
                                eprintln!("{}", e);
                                return;
                            }

                            let mut local_reader = BufReader::new(&stream);
                            let mut local_writer = BufWriter::new(&stream);

                            loop {
                                match Request::from_reader(&mut local_reader) {
                                    Ok(mut request) => {
                                        if let Err(e) = forward_request(
                                            &*name,
                                            &mut request,
                                            &mut local_writer,
                                            &remote_address,
                                            timeout,
                                        ) {
                                            eprintln!("{}", e);
                                        }
                                    }
                                    Err(e) => {
                                        if e != TcpIpError::TcpTimeout {
                                            eprintln!("{}", e);
                                        }
                                        break;
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            }
        }))
    }
}

impl FromStr for Server {
    type Err = TcpIpError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut items = s.split(' ');

        let listen_port = items
            .next()
            .ok_or_else(|| TcpIpError::new("Config - Missing listen port"))?
            .parse()
            .map_err(|e| TcpIpError::new(format!("Failed to read listen port - {}", e)))?;

        let remote_address = items
            .next()
            .ok_or_else(|| TcpIpError::new("Config - Missing remote address"))?
            .parse()
            .map_err(|e| TcpIpError::new(format!("Failed to read remote address - {}", e)))?;

        let timeout = items
            .next()
            .unwrap_or("4")
            .parse()
            .map_err(|e| TcpIpError::new(format!("Failed to read timeout - {}", e)))?;

        let name = format!("{} -> {}", listen_port, remote_address);

        Ok(Self {
            listen_port,
            remote_address,
            timeout,
            name,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddrV4;
    use std::str::FromStr;

    use crate::config::Server;

    #[test]
    fn from_str_server() {
        let config = r#"80 127.0.0.1:5000"#;

        let c = Server::from_str(config).expect("Failed to parse server");

        assert_eq!(c.listen_port, 80);
        assert_eq!(
            c.remote_address,
            SocketAddrV4::from_str("127.0.0.1:5000").expect("Failed to parse address")
        );
    }
}
