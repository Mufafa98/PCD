use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
};

use crate::{
    net_app::NetResult,
    transport::{
        NetworkCommunicationChannel, NetworkListener, NetworkProtocol, TransferStats,
        TransferStatsBuilder,
    },
};

pub struct Tcp {}
pub struct TcpListenerWrapper {
    listner: TcpListener,
}
pub struct TcpCommunicationChannel {
    stream: TcpStream,
}

impl NetworkProtocol for Tcp {
    fn connect() -> NetResult<Box<dyn NetworkCommunicationChannel>> {
        Ok(Box::new(TcpCommunicationChannel {
            stream: TcpStream::connect("127.0.0.1:8080")?,
        }))
    }

    fn bind() -> NetResult<Box<dyn NetworkListener>> {
        Ok(Box::new(TcpListenerWrapper {
            listner: TcpListener::bind("127.0.0.1:8080")?,
        }))
    }
}

impl NetworkListener for TcpListenerWrapper {
    fn accept(
        self: Box<TcpListenerWrapper>,
    ) -> NetResult<(Box<dyn NetworkCommunicationChannel>, Option<SocketAddr>)> {
        let (comm, addr) = self.listner.accept()?;
        Ok((
            Box::new(TcpCommunicationChannel { stream: comm }),
            Some(addr),
        ))
    }
}

impl NetworkCommunicationChannel for TcpCommunicationChannel {
    fn read(&mut self, buffer: &mut [u8]) -> Result<TransferStats, std::io::Error> {
        self.stream.read_exact(buffer)?;
        Ok(TransferStatsBuilder::new().received(buffer.len()).build())
    }

    fn write(
        &mut self,
        buffer: &[u8],
        _: Option<SocketAddr>,
    ) -> Result<TransferStats, std::io::Error> {
        self.stream.write_all(buffer)?;
        self.stream.flush()?;
        Ok(TransferStatsBuilder::new().sent(buffer.len()).build())
    }

    fn sw_read(&mut self, buffer: &mut [u8]) -> Result<TransferStats, std::io::Error> {
        self.read(buffer)
    }

    fn sw_write(
        &mut self,
        buffer: &[u8],
        to: Option<SocketAddr>,
    ) -> Result<TransferStats, std::io::Error> {
        self.write(buffer, to)
    }
}
