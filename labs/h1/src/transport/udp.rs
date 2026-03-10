use std::{
    io::ErrorKind,
    net::{SocketAddr, UdpSocket},
    time::Duration,
};

use crate::{
    net_app::NetResult,
    transport::{
        NetworkCommunicationChannel, NetworkListener, NetworkProtocol, TransferStats,
        TransferStatsBuilder,
    },
};

pub struct Udp {}
pub struct UdpListenerWrapper {
    listner: UdpSocket,
}
pub struct UdpCommunicationChannel {
    stream: UdpSocket,
}

impl NetworkProtocol for Udp {
    fn connect() -> NetResult<Box<dyn NetworkCommunicationChannel>> {
        Ok(Box::new(UdpCommunicationChannel {
            stream: UdpSocket::bind("127.0.0.1:0")?,
        }))
    }

    fn bind() -> NetResult<Box<dyn NetworkListener>> {
        Ok(Box::new(UdpListenerWrapper {
            listner: UdpSocket::bind("127.0.0.1:8080")?,
        }))
    }
}

impl NetworkListener for UdpListenerWrapper {
    fn accept(
        self: Box<UdpListenerWrapper>,
    ) -> NetResult<(Box<dyn NetworkCommunicationChannel>, Option<SocketAddr>)> {
        self.listner
            .set_read_timeout(Some(Duration::from_millis(50)))?;
        Ok((
            Box::new(UdpCommunicationChannel {
                stream: self.listner.try_clone()?,
            }),
            None,
        ))
    }
}

impl NetworkCommunicationChannel for UdpCommunicationChannel {
    fn read(&mut self, buffer: &mut [u8]) -> Result<TransferStats, std::io::Error> {
        match self.stream.recv_from(buffer) {
            Ok((n, addr)) => Ok(TransferStatsBuilder::new().addr(addr).received(n).build()),
            Err(e) => Err(e),
        }
    }

    fn write(
        &mut self,
        buffer: &[u8],
        to: Option<SocketAddr>,
    ) -> Result<TransferStats, std::io::Error> {
        let addr = to.unwrap();
        let n = self.stream.send_to(buffer, addr)?;

        Ok(TransferStatsBuilder::new().addr(addr).sent(n).build())
    }

    fn sw_read(&mut self, buffer: &mut [u8]) -> Result<TransferStats, std::io::Error> {
        let mut stats = TransferStats::empty();

        loop {
            match self.read(buffer) {
                Ok(read_stat) => {
                    stats.merge(&read_stat);

                    if let Some(client_addr) = stats.address {
                        self.stream.send_to(&[1u8], client_addr)?;
                        stats.sent(1);
                    }
                    return Ok(stats);
                }
                Err(e) => match e.kind() {
                    ErrorKind::WouldBlock | ErrorKind::TimedOut => continue,
                    _ => return Err(e),
                },
            }
        }
    }

    fn sw_write(
        &mut self,
        buffer: &[u8],
        to: Option<SocketAddr>,
    ) -> Result<TransferStats, std::io::Error> {
        let addr = to.unwrap();
        let mut ack_signal = [0u8; 1];
        let mut stats = TransferStatsBuilder::new().addr(addr).build();

        let mut is_first_send = true;

        loop {
            let n = self.stream.send_to(buffer, addr)?;

            if is_first_send {
                stats.sent(n);
                is_first_send = false;
            } else {
                stats.resent(n);
            }

            match self.stream.recv_from(&mut ack_signal) {
                Ok((n, _)) => {
                    stats.received(n);

                    if ack_signal[0] == 1 && n == 1 {
                        break;
                    }
                }
                Err(e) => match e.kind() {
                    ErrorKind::WouldBlock | ErrorKind::TimedOut => continue,
                    _ => return Err(e),
                },
            }
        }
        Ok(stats)
    }
}
