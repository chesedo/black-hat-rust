mod common_ports;
mod error;
mod models;
mod ports;
mod subdomains;

use std::{env, time::Duration};

use anyhow::Result;
use error::Error;
use futures::{stream, StreamExt};
use models::Subdomain;
use reqwest::{redirect, Client};

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        return Err(Error::CliUsage.into());
    }

    let target = &args[1];

    let http_timeout = Duration::from_secs(5);
    let http_client = Client::builder()
        .redirect(redirect::Policy::limited(4))
        .timeout(http_timeout)
        .build()?; // we use a custom threadpool to improve speed

    let ports_concurrency = 200;
    let subdomains_concurrency = 100;

    let subdomains = subdomains::enumerate(&http_client, target).await?;
    let scan_result: Vec<Subdomain> = stream::iter(subdomains.into_iter())
        .map(|subdomain| ports::scan_ports(ports_concurrency, subdomain))
        .buffer_unordered(subdomains_concurrency)
        .collect()
        .await;

    for subdomain in scan_result {
        println!("{}:", &subdomain.domain);

        for port in &subdomain.open_ports {
            println!(" {}", port.port);
        }

        println!();
    }

    println!("Done!!");
    Ok(())
}
