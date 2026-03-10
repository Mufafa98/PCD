use std::io::Write;
use std::{
    fs::File,
    sync::{Arc, Barrier},
    thread,
};

use crate::{
    config::{ConfigBuilder, Protocol, TransferMechanism},
    log::Logger,
    net_app::{client, server},
    payload::size::Size,
};

mod cli;
mod config;
mod log;
mod net_app;
mod payload;
mod transport;

use clap::Parser;
use cli::Cli;

fn normal_run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let config = Arc::new(
        ConfigBuilder::new()
            .protocol(cli.protocol)
            .transfer_mechanism(cli.mechanism)
            .payload_size(cli.payload)
            .block_size(cli.batch)
            .seed(cli.seed)
            .build(),
    );

    let client_logger = Logger::new("client_log.txt")?;
    let server_logger = Logger::new("server_log.txt")?;

    let barrier = Arc::new(Barrier::new(2));

    let server_handle = thread::spawn({
        let config = Arc::clone(&config);
        let barrier = Arc::clone(&barrier);
        move || server::run(server_logger, barrier, &config)
    });
    let client_handle = thread::spawn({
        let config = Arc::clone(&config);
        let barrier = Arc::clone(&barrier);
        move || client::run(client_logger, barrier, &config)
    });

    let (rcv_m, rcv_b) = server_handle
        .join()
        .map_err(|e| format!("Thread panicked: {:?}", e))?
        .map_err(|e| format!("Server error: {}", e))?;
    let (t, sent_m, sent_b) = client_handle
        .join()
        .map_err(|e| format!("Thread panicked: {:?}", e))?
        .map_err(|e| format!("Client error: {}", e))?;
    println!(
        "Sent M {} Received M {} Sent B {} Received B {} Time {} ms",
        sent_m,
        rcv_m,
        sent_b,
        rcv_b,
        t / 1000
    );
    Ok(())
}

fn generate_results(_: Cli) -> Result<(), Box<dyn std::error::Error>> {
    use itertools::iproduct;

    let protocols = [Protocol::Tcp, Protocol::Udp, Protocol::Quic];
    let transfers = [TransferMechanism::Streaming, TransferMechanism::StopAndWait];
    let payloads = [Size::MB(500), Size::GB(1)];
    let blocks = [
        Size::Byte(500),
        Size::KB(1),
        Size::KB(10),
        Size::KB(30),
        Size::KB(60),
    ];

    let tries = 30;
    let mut csv_file = File::create("results.csv")?;
    writeln!(
        csv_file,
        "Protocol,Transfer,Payload,Block,Trial,Server1,Server2,Client1,Client2,Client3"
    )?;

    let iterator = iproduct!(
        protocols.iter(),
        transfers.iter(),
        payloads.iter(),
        blocks.iter(),
        0..tries
    );

    for (protocol, transfer, payload, block, trie) in iterator {
        let config = Arc::new(
            ConfigBuilder::new()
                .protocol(protocol.clone())
                .transfer_mechanism(transfer.clone())
                .payload_size(*payload)
                .block_size(*block)
                .seed(31415)
                .build(),
        );

        let client_log = format!(
            "out/client_{:?}_{:?}_{:?}_{:?}.txt",
            protocol, transfer, payload, block
        );
        let server_log = format!(
            "out/server_{:?}_{:?}_{:?}_{:?}.txt",
            protocol, transfer, payload, block
        );

        let client_logger = Logger::new(&client_log)?;
        let server_logger = Logger::new(&server_log)?;

        let barrier = Arc::new(Barrier::new(2));

        let server_handle = thread::spawn({
            let config = Arc::clone(&config);
            let barrier = Arc::clone(&barrier);
            move || server::run(server_logger, barrier, &config)
        });
        let client_handle = thread::spawn({
            let config = Arc::clone(&config);
            let barrier = Arc::clone(&barrier);
            move || client::run(client_logger, barrier, &config)
        });

        let server_res = server_handle
            .join()
            .map_err(|e| format!("Thread panicked: {:?}", e))?
            .map_err(|e| format!("Server error: {}", e))?;

        let client_res = client_handle
            .join()
            .map_err(|e| format!("Thread panicked: {:?}", e))?
            .map_err(|e| format!("Client error: {}", e))?;

        writeln!(
            csv_file,
            "{:?},{:?},{:?},{:?},{},{},{},{},{},{}",
            protocol,
            transfer,
            payload,
            block,
            trie,
            server_res.0,
            server_res.1,
            client_res.0,
            client_res.1,
            client_res.2
        )?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if !cli.generate_results {
        normal_run(cli)?;
    } else {
        generate_results(cli)?;
    }
    Ok(())
}
