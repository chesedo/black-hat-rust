use futures::StreamExt;
use tokio::{net::TcpStream, sync::mpsc};

use crate::{
    common_ports::MOST_COMMON_PORTS_100,
    models::{Port, Subdomain},
};
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;

pub async fn scan_ports(concurrency: usize, mut subdomain: Subdomain) -> Subdomain {
    let mut socket_addresses = format!("{}:1024", subdomain.domain)
        .to_socket_addrs()
        .expect("port scanner: Creating socket address");

    if let Some(socket_address) = socket_addresses.next() {
        // Concurrent stream method 3: using channels
        let (input_tx, input_rx) = mpsc::channel(concurrency);
        let (output_tx, output_rx) = mpsc::channel(concurrency);
        tokio::spawn(async move {
            for port in MOST_COMMON_PORTS_100 {
                let _ = input_tx.send(*port).await;
            }
        });

        let input_rx_stream = tokio_stream::wrappers::ReceiverStream::new(input_rx);
        input_rx_stream
            .for_each_concurrent(concurrency, |port| {
                let output_tx = output_tx.clone();

                async move {
                    let port = scan_port(socket_address, port).await;
                    if port.is_open {
                        let _ = output_tx.send(port).await;
                    }
                }
            })
            .await;
        // close channel
        drop(output_tx);

        let output_rx_stream = tokio_stream::wrappers::ReceiverStream::new(output_rx);
        subdomain.open_ports = output_rx_stream.collect().await;
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

    Port { port, is_open }
}
