use std::str::FromStr;

use clap::Parser;

use crate::{config, payload::size::Size};

#[derive(Parser)]
pub struct Cli {
    #[arg(short, long, value_enum, default_value = "tcp")]
    pub protocol: config::Protocol,

    #[arg(short, long, value_enum, default_value = "streaming")]
    pub mechanism: config::TransferMechanism,

    #[arg(long, default_value = "314159")]
    pub seed: u64,

    #[arg(short = 's', long, default_value = "500MB", value_parser = parse_payload_size)]
    pub payload: Size,

    #[arg(short = 'b', long, default_value = "60KB", value_parser = parse_batch_size)]
    pub batch: Size,

    #[arg(short, long, default_value = "false")]
    pub generate_results: bool,
}

fn parse_batch_size(s: &str) -> Result<Size, String> {
    let size = Size::from_str(s)?;

    if size.to_bytes() >= 64 * 1024 {
        Err("Batch size must be 64KB or less".to_string())
    } else {
        Ok(size)
    }
}

fn parse_payload_size(s: &str) -> Result<Size, String> {
    let size = Size::from_str(s)?;

    if size.to_bytes() > Size::GB(1).to_bytes() {
        Err("Payload size must be 1GB or less".to_string())
    } else {
        Ok(size)
    }
}
