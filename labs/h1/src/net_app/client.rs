use std::io::Write;
use std::time::Instant;
use std::{
    net::SocketAddr,
    sync::{Arc, Barrier},
};

use crate::transport::TransferStats;
use crate::transport::packet::{NetworkSerialize, Packet, PacketType};
use crate::transport::quic::Quic;
use crate::{
    config::{Config, TransferMechanism},
    log::Logger,
    log_to,
    net_app::NetResult,
    payload::generator::Payload,
    transport::{NetworkProtocol, tcp::Tcp, udp::Udp},
};

pub fn run(
    mut l: Logger,
    barrier: Arc<Barrier>,
    config: &Config,
) -> NetResult<(u128, usize, usize)> {
    // Prepare Stats
    let mut snd_message_count = 0;
    let mut snd_bytes_count = 0;
    let mut update_stats = |stats: &TransferStats| {
        snd_message_count += stats.total_messages_sent();
        snd_bytes_count += stats.total_bytes_sent();
    };

    log_to!(l, "Preparing payload");
    let payload = Payload::new(config.payload_size(), config.seed());
    let chunks = payload.chunks(config.block_size().to_bytes());
    log_to!(l, "Payload succesfully generated");
    // Wait for server to start and establish a connection
    barrier.wait();
    let mut stream = match config.protocol() {
        crate::config::Protocol::Tcp => Tcp::connect()?,
        crate::config::Protocol::Udp => Udp::connect()?,
        crate::config::Protocol::Quic => Quic::connect()?,
    };

    let server_addr: Option<SocketAddr> = match config.protocol() {
        crate::config::Protocol::Udp => Some("127.0.0.1:8080".parse().unwrap()),
        crate::config::Protocol::Tcp => None,
        crate::config::Protocol::Quic => None,
    };
    log_to!(l, "Succesfully connected to server");

    log_to!(l, "Started sending chunks");
    let start = Instant::now();
    for chunk in chunks {
        let buffer = Packet::new(PacketType::Payload, chunk).to_bytes();
        let stats = match config.transfer_mechanism() {
            TransferMechanism::Streaming => stream.write(&buffer, server_addr)?,
            TransferMechanism::StopAndWait => stream.sw_write(&buffer, server_addr)?,
        };
        update_stats(&stats);
    }

    let buffer = Packet::new(PacketType::EndPayload, vec![1u8; 1]).to_bytes();
    let stats = match config.transfer_mechanism() {
        TransferMechanism::Streaming => stream.write(&buffer, server_addr)?,
        TransferMechanism::StopAndWait => stream.sw_write(&buffer, server_addr)?,
    };
    update_stats(&stats);
    log_to!(l, "Finished sending chunks");

    let mut buffer = [0u8; 64];
    let stats = stream.sw_read(&mut buffer)?;
    update_stats(&stats);

    log_to!(l, "Received final hash");

    let received_hash = String::from_utf8_lossy(&buffer).to_string();
    let payload_hash = payload.hash();

    let hash_status = if payload_hash != received_hash {
        [0u8; 1]
    } else {
        [1u8; 1]
    };
    let stats = stream.sw_write(&hash_status, server_addr)?;
    let elapsed = start.elapsed();
    update_stats(&stats);

    log_to!(l, "Confirmation sent");

    let transfer_status = if hash_status[0] == 0 {
        "Fail"
    } else {
        "Success"
    };

    log_to!(
        l,
        "[{}] Total transmision time: [{:.3?}] Messages received: [{}] Bytes received: [{}]",
        transfer_status,
        elapsed,
        snd_message_count,
        snd_bytes_count
    );
    Ok((elapsed.as_micros(), snd_message_count, snd_bytes_count))
}
