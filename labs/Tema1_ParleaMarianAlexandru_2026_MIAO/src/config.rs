use crate::payload::size::Size;
use clap::ValueEnum;

#[derive(Debug, PartialEq, Clone, ValueEnum)]
pub enum Protocol {
    Tcp,
    Udp,
    Quic,
}

#[derive(Debug, PartialEq, Clone, ValueEnum)]
pub enum TransferMechanism {
    Streaming,
    StopAndWait,
}

pub struct Config {
    protocol: Protocol,
    payload_size: Size,
    transfer_mechanism: TransferMechanism,
    block_size: Size,
    seed: u64,
}

macro_rules! getter {
    ($name:ident, $type:ty) => {
        pub fn $name(&self) -> &$type {
            &self.$name
        }
    };
}

impl Config {
    getter!(protocol, Protocol);
    getter!(payload_size, Size);
    getter!(transfer_mechanism, TransferMechanism);
    getter!(block_size, Size);
    getter!(seed, u64);
}
#[macro_export]
macro_rules! builder_method {
    ($name:ident, $type:ty) => {
        pub fn $name(mut self, value: $type) -> Self {
            self.0.$name = value;
            self
        }
    };
}

pub struct ConfigBuilder(Config);

impl ConfigBuilder {
    pub fn new() -> Self {
        Self(Config {
            protocol: Protocol::Tcp,
            payload_size: Size::Byte(1),
            transfer_mechanism: TransferMechanism::Streaming,
            block_size: Size::Byte(1),
            seed: 314,
        })
    }

    builder_method!(protocol, Protocol);
    builder_method!(payload_size, Size);
    builder_method!(transfer_mechanism, TransferMechanism);
    pub fn block_size(mut self, size: Size) -> Self {
        if size.to_bytes() > Size::KB(64).to_bytes() {
            panic!("Not safe to use block size greater than 64KB")
        }
        self.0.block_size = size;
        self
    }
    builder_method!(seed, u64);

    pub fn build(self) -> Config {
        self.0
    }
}
