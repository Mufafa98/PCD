use std::{error::Error, fmt::Display};

pub mod client;
pub mod server;

#[derive(Debug)]
#[allow(unused)]
pub enum NetError {
    HashVerificationFailed,
    QuicWriteError,
}

impl Display for NetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for NetError {}
unsafe impl Send for NetError {}
unsafe impl Sync for NetError {}

pub type NetResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
