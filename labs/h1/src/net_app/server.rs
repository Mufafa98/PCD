use std::io::Write;
use std::sync::{Arc, Barrier};

use crate::config::Protocol;
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

pub fn run(mut l: Logger, barrier: Arc<Barrier>, config: &Config) -> NetResult<(usize, usize)> {
    let mut rcv_message_count = 0;
    let mut rcv_bytes_count = 0;
    let mut update_stats = |stats: &TransferStats| {
        rcv_message_count += stats.messages_received;
        rcv_bytes_count += stats.bytes_received;
    };

    let (mut stream, mut addr) = match config.protocol() {
        Protocol::Tcp => {
            let listener = Tcp::bind()?;
            barrier.wait();
            listener.accept()?
        }
        Protocol::Udp => {
            let listner = Udp::bind()?;
            barrier.wait();
            listner.accept()?
        }
        Protocol::Quic => {
            let listener = Quic::bind()?;
            barrier.wait();
            listener.accept()?
        }
    };
    log_to!(l, "Succesfully established connection to {:?}", addr);

    let payload_size = config.payload_size().to_bytes();
    let block_size = config.block_size().to_bytes();
    let flag_size = size_of::<PacketType>();

    let chunks_to_read = (payload_size as f64 / block_size as f64).ceil() as usize;
    let last_block_size = if payload_size.is_multiple_of(block_size) {
        block_size
    } else {
        payload_size % block_size
    };

    let mut chunk_buffer: Vec<u8> = vec![0u8; flag_size + block_size];
    let mut payload = Payload::empty(*config.payload_size());

    log_to!(l, "Started receiving chunks");
    let mut counter = 0;
    loop {
        if *config.protocol() != Protocol::Udp {
            if counter == chunks_to_read - 1 {
                chunk_buffer.resize(flag_size + last_block_size, 0);
            } else if counter == chunks_to_read {
                chunk_buffer.resize(flag_size + 1, 0);
            };
        }

        let stats = match config.transfer_mechanism() {
            TransferMechanism::Streaming => stream.read(&mut chunk_buffer)?,
            TransferMechanism::StopAndWait => stream.sw_read(&mut chunk_buffer)?,
        };
        addr = addr.or(stats.address);
        update_stats(&stats);

        let packet = Packet::<Vec<u8>>::from_bytes(&chunk_buffer)?;
        match packet.packet_type {
            PacketType::Payload => payload.extend_from_bytes(&packet.data),
            PacketType::EndPayload => break,
        }

        counter += 1;
    }
    log_to!(l, "Finished receiving chunks");
    let hash = payload.hash();
    let buffer = hash.as_bytes();

    log_to!(l, "Sending final hash, peer_addr: {:?}", addr);
    let stats = stream.sw_write(buffer, addr)?;
    update_stats(&stats);

    log_to!(l, "Receiving confirmation");
    let mut buffer = [0; 1];
    let stats = stream.sw_read(&mut buffer)?;
    update_stats(&stats);

    let transfer_status = if buffer[0] == 1 { "Success" } else { "Fail" };

    log_to!(
        l,
        "[{}] Protocol: [{:?}] Messages received: [{}] Bytes received: [{}]",
        transfer_status,
        config.protocol(),
        rcv_message_count,
        rcv_bytes_count
    );
    Ok((rcv_message_count, rcv_bytes_count))
}
