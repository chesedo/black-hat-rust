use futures::{stream, StreamExt};
use tokio::net::TcpStream;

use crate::{
    common_ports::MOST_COMMON_PORTS_100,
    modules::{Port, Subdomain},
};
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;

pub async fn scan_ports(concurrency: usize, mut subdomain: Subdomain) -> Subdomain {
    let mut socket_addresses = format!("{}:1024", subdomain.domain)
        .to_socket_addrs()
        .expect("port scanner: Creating socket address");

    if let Some(socket_address) = socket_addresses.next() {
        subdomain.open_ports = stream::iter(MOST_COMMON_PORTS_100.iter())
            .map(|port| async move {
                let port = scan_port(socket_address, *port).await;
                if port.is_open {
                    return Some(port);
                }
                None
            })
            .buffer_unordered(concurrency)
            .filter_map(|port| async { port })
            .collect()
            .await;
    }

    subdomain
}

async fn scan_port(mut socket_address: SocketAddr, port: u16) -> Port {
    let timeout = Duration::from_secs(3);
    socket_address.set_port(port);

    let is_open = matches!(
        tokio::time::timeout(timeout, TcpStream::connect(&socket_address)).await,
        Ok(Ok(_))
    );

    Port {
        port,
        is_open,
        findings: Vec::new(),
    }
}
