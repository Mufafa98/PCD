use std::net::SocketAddr;

use crate::net_app::NetResult;

pub mod packet;
pub mod quic;
pub mod tcp;
pub mod udp;

#[derive(Debug, Clone)]
pub struct TransferStats {
    pub bytes_sent: usize,
    pub messages_sent: usize,

    pub bytes_resent: usize,
    pub messages_resent: usize,

    pub bytes_received: usize,
    pub messages_received: usize,

    pub address: Option<SocketAddr>,
}

impl TransferStats {
    pub fn empty() -> Self {
        Self {
            bytes_sent: 0,
            messages_sent: 0,

            bytes_resent: 0,
            messages_resent: 0,

            bytes_received: 0,
            messages_received: 0,

            address: None,
        }
    }

    pub fn sent(&mut self, bytes: usize) {
        self.messages_sent += 1;
        self.bytes_sent += bytes;
    }

    pub fn resent(&mut self, bytes: usize) {
        self.messages_resent += 1;
        self.bytes_resent += bytes;
    }

    pub fn received(&mut self, bytes: usize) {
        self.messages_received += 1;
        self.bytes_received += bytes;
    }

    pub fn merge(&mut self, other: &TransferStats) {
        self.bytes_sent += other.bytes_sent;
        self.messages_sent += other.messages_sent;
        self.bytes_resent += other.bytes_resent;
        self.messages_resent += other.messages_resent;
        self.bytes_received += other.bytes_received;
        self.messages_received += other.messages_received;

        if self.address.is_none() && other.address.is_some() {
            self.address = other.address;
        }
    }

    pub fn total_bytes_sent(&self) -> usize {
        self.bytes_sent + self.bytes_resent
    }

    pub fn total_messages_sent(&self) -> usize {
        self.messages_sent + self.messages_resent
    }
}

struct TransferStatsBuilder(TransferStats);
impl TransferStatsBuilder {
    pub fn new() -> Self {
        Self(TransferStats::empty())
    }
    pub fn addr(mut self, addr: SocketAddr) -> Self {
        self.0.address = Some(addr);
        self
    }
    pub fn sent(mut self, bytes: usize) -> Self {
        self.0.sent(bytes);
        self
    }

    pub fn received(mut self, bytes: usize) -> Self {
        self.0.received(bytes);
        self
    }

    pub fn build(self) -> TransferStats {
        self.0
    }
}

pub trait NetworkProtocol {
    fn connect() -> NetResult<Box<dyn NetworkCommunicationChannel>>;
    fn bind() -> NetResult<Box<dyn NetworkListener>>;
}

pub trait NetworkListener {
    fn accept(
        self: Box<Self>,
    ) -> NetResult<(Box<dyn NetworkCommunicationChannel>, Option<SocketAddr>)>;
}

pub trait NetworkCommunicationChannel {
    fn read(&mut self, buffer: &mut [u8]) -> Result<TransferStats, std::io::Error>;
    fn write(
        &mut self,
        buffer: &[u8],
        to: Option<SocketAddr>,
    ) -> Result<TransferStats, std::io::Error>;
    // Stop and wait
    fn sw_read(&mut self, buffer: &mut [u8]) -> Result<TransferStats, std::io::Error>;
    fn sw_write(
        &mut self,
        buffer: &[u8],
        to: Option<SocketAddr>,
    ) -> Result<TransferStats, std::io::Error>;
}
