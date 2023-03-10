use futures::stream;
use futures::StreamExt;
use reqwest::Client;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::time::Duration;
use std::time::Instant;
use tracing::debug;
use tracing::error;
use tracing::info;

use crate::dns;
use crate::modules::Module;
use crate::modules::Subdomain;
use crate::ports;
use crate::{modules, Error};

pub fn modules() {
    print("Subdomains modules", modules::all_subdomains_modules());
    print("HTTP modules", modules::all_http_modules());
}

fn print<M: Module + ?Sized>(heading: &str, modules: Vec<Box<M>>) {
    println!("{heading}");
    for module in modules {
        println!("   {}: {}", module.name(), module.description());
    }
}

pub fn scan(target: &str) -> Result<(), Error> {
    info!("Scanning: {}", target);

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Building tokio's runtime");

    let http_timeout = Duration::from_secs(10);
    let http_client = Client::builder().timeout(http_timeout).build()?;
    let dns_resolver = dns::new_resolver();

    let subdomains_concurrency = 20;
    let dns_concurrency = 100;
    let ports_concurrency = 200;
    let vulnerabilities_conccurency = 20;
    let scan_start = Instant::now();

    let subdomains_modules = modules::all_subdomains_modules();

    runtime.block_on(async move {
        // 1st step: concurrently scan subdomains
        let mut subdomains: Vec<String> = stream::iter(subdomains_modules.into_iter())
            .map(|module| async move {
                match module.enumerate(target).await {
                    Ok(new_subdomains) => Some(new_subdomains),
                    Err(err) => {
                        error!("subdomains/{}: {}", module.name(), err);
                        None
                    }
                }
            })
            .buffer_unordered(subdomains_concurrency)
            .filter_map(|domain| async { domain })
            .collect::<Vec<Vec<String>>>()
            .await
            .into_iter()
            .flatten()
            .collect();

        subdomains.push(target.to_string());

        // 2nd step: dedup, clean and convert results
        let subdomains: Vec<Subdomain> = HashSet::<String>::from_iter(subdomains.into_iter())
            .into_iter()
            .filter(|subdomain| subdomain.contains(target))
            .map(|domain| Subdomain {
                domain,
                open_ports: Vec::new(),
            })
            .collect();

        info!("Found {} domains", subdomains.len());

        // 3rd step: concurrently filter unresolvable domains
        let subdomains: Vec<Subdomain> = stream::iter(subdomains.into_iter())
            .map(|domain| dns::resolves(&dns_resolver, domain))
            .buffer_unordered(dns_concurrency)
            .filter_map(|domain| async move { domain })
            .collect()
            .await;

        // 4th step: concurrently scan ports
        let subdomains: Vec<Subdomain> = stream::iter(subdomains.into_iter())
            .map(|domain| {
                info!("Scannig ports for {}", &domain.domain);
                ports::scan_ports(ports_concurrency, domain)
            })
            .buffer_unordered(1)
            .collect()
            .await;

        for subdomain in &subdomains {
            println!("{}", subdomain.domain);
            for port in &subdomain.open_ports {
                println!("    {}", port.port);
            }
        }

        println!("---------------------- Vulnerabilities ----------------------");

        // 5th step: concurrently scan vulnerabilities
        stream::iter(subdomains.into_iter())
            .map(|domain| {
                domain
                    .open_ports
                    .iter()
                    .map(|port| format!("http://{}:{}", &domain.domain, port.port))
                    .collect::<Vec<String>>()
            })
            .map(|targets| {
                stream::iter(targets.into_iter().map(|target| {
                    let http_modules = modules::all_http_modules();
                    return (http_modules, target);
                }))
            })
            .flatten()
            .map(|(modules, target)| {
                stream::iter(modules.into_iter().map(move |module| {
                    return (module, target.clone());
                }))
            })
            .flatten()
            .for_each_concurrent(vulnerabilities_conccurency, |(module, target)| {
                let http_client = http_client.clone();
                async move {
                    info!("Running {} for {}", module.name(), target);
                    match module.scan(&http_client, &target).await {
                        Ok(Some(finding)) => println!("{:?}", &finding),
                        Ok(None) => {}
                        Err(err) => debug!("Error: {}", err),
                    };
                }
            })
            .await;
    });

    let scan_duration = scan_start.elapsed();
    info!("Scan completed in {:?}", scan_duration);

    Ok(())
}
