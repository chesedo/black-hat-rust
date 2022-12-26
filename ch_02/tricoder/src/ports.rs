use crate::{
    common_ports::MOST_COMMON_PORTS_100,
    models::{Port, Subdomain},
};
use rayon::prelude::*;
use std::net::{SocketAddr, ToSocketAddrs};
use std::{net::TcpStream, time::Duration};

pub fn scan_ports(mut subdomain: Subdomain) -> Subdomain {
    let mut socket_addresses = format!("{}:1024", subdomain.domain)
        .to_socket_addrs()
        .expect("port scanner: Creating socket address");

    if let Some(socket_address) = socket_addresses.next() {
        subdomain.open_ports = MOST_COMMON_PORTS_100
            .into_par_iter()
            .map(|port| scan_port(socket_address, *port))
            .filter(|port| port.is_open) // filter closed ports
            .collect();
    }

    subdomain
}

fn scan_port(mut socket_address: SocketAddr, port: u16) -> Port {
    let timeout = Duration::from_secs(3);
    socket_address.set_port(port);

    let is_open = TcpStream::connect_timeout(&socket_address, timeout).is_ok();

    Port { port, is_open }
}
