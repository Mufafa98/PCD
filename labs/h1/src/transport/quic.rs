use std::{
    io::ErrorKind,
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
    time::Duration,
};

use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::{
    net_app::NetResult,
    payload::size::Size,
    transport::{NetworkCommunicationChannel, NetworkListener, NetworkProtocol, TransferStats},
};

pub const SERVER_IP: &str = "127.0.0.1:8080";
pub const MAX_DATAGRAM_SIZE: usize = 65536;
pub const STREAM_ID: u64 = 0;

use std::cell::RefCell;

thread_local! {
    static OUT_BUF: RefCell<[u8; MAX_DATAGRAM_SIZE]> = const { RefCell::new([0; MAX_DATAGRAM_SIZE]) };
}

pub fn flush_egress(socket: &UdpSocket, conn: &mut quiche::Connection) {
    OUT_BUF.with(|buf| {
        let mut out = buf.borrow_mut();
        while let Ok((write, send_info)) = conn.send(&mut *out) {
            let _ = socket.send_to(&out[..write], send_info.to);
        }
    });
}

fn map_quiche_err(err: quiche::Error) -> std::io::Error {
    std::io::Error::other(err.to_string())
}

pub struct Quic {}
pub struct QuicListenerWrapper {
    socket: UdpSocket,
    config: quiche::Config,
}
pub struct QuicCommunicationChannel {
    socket: UdpSocket,
    conn: quiche::Connection,
    stream_id: u64,
}

fn get_config() -> Result<quiche::Config, quiche::Error> {
    let max_data = Size::GB(2).to_bytes() as u64;
    let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION)?;
    config.set_application_protos(&[b"\x0ahq-interop"])?;
    config.set_initial_max_data(max_data);
    config.set_initial_max_stream_data_bidi_local(max_data);
    config.set_initial_max_stream_data_bidi_remote(max_data);
    config.set_initial_max_streams_bidi(5);
    config.set_max_recv_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_max_send_udp_payload_size(MAX_DATAGRAM_SIZE);

    config.set_cc_algorithm(quiche::CongestionControlAlgorithm::Reno); // For some reason better than default cubic

    Ok(config)
}

fn drain_incoming(socket: &UdpSocket, conn: &mut quiche::Connection) -> std::io::Result<bool> {
    let mut buf = [0; MAX_DATAGRAM_SIZE];
    let to = socket.local_addr()?;
    let mut got_data = false;
    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, from)) => {
                let recv_info = quiche::RecvInfo { to, from };
                let _ = conn.recv(&mut buf[..len], recv_info);
                got_data = true;
            }
            Err(e) => match e.kind() {
                ErrorKind::WouldBlock => return Ok(got_data),
                _ => return Err(e),
            },
        }
    }
}

fn wait_for_data(socket: &UdpSocket, conn: &mut quiche::Connection) -> std::io::Result<()> {
    use std::time::Instant;
    let timeout = conn
        .timeout()
        .unwrap_or(Duration::from_millis(5))
        .max(Duration::from_micros(100));

    let deadline = Instant::now() + timeout;
    let local = socket.local_addr()?;
    let mut buf = [0; MAX_DATAGRAM_SIZE];

    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, from)) => {
                let recv_info = quiche::RecvInfo { to: local, from };
                let _ = conn.recv(&mut buf[..len], recv_info);
                drain_incoming(socket, conn)?;
                return Ok(());
            }
            Err(e) => match e.kind() {
                ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        conn.on_timeout();
                        return Ok(());
                    }
                }
                _ => return Err(e),
            },
        }
    }
}

impl NetworkProtocol for Quic {
    fn connect() -> NetResult<Box<dyn NetworkCommunicationChannel>> {
        let mut config = get_config()?;
        config.verify_peer(false);

        let server_addr = SERVER_IP.to_socket_addrs()?.next().unwrap();
        let socket = UdpSocket::bind("0.0.0.0:0")?;

        socket.set_nonblocking(true)?;
        
        let mut scid = [0; quiche::MAX_CONN_ID_LEN];
        StdRng::seed_from_u64(314).fill_bytes(&mut scid);
        let scid = quiche::ConnectionId::from_ref(&scid);
        let local_addr = socket.local_addr()?;

        let mut conn = quiche::connect(None, &scid, local_addr, server_addr, &mut config)?;
        flush_egress(&socket, &mut conn);

        while !conn.is_established() && !conn.is_closed() {
            wait_for_data(&socket, &mut conn)?;
            flush_egress(&socket, &mut conn);
        }
        if conn.is_closed() {
            return Err("Connection closed during handshake".into());
        }

        Ok(Box::new(QuicCommunicationChannel {
            socket,
            conn,
            stream_id: STREAM_ID,
        }))
    }

    fn bind() -> NetResult<Box<dyn NetworkListener>> {
        let mut config = get_config()?;
        config.load_cert_chain_from_pem_file("server.crt")?;
        config.load_priv_key_from_pem_file("server.key")?;

        let socket = UdpSocket::bind(SERVER_IP)?;

        socket.set_nonblocking(true)?;

        Ok(Box::new(QuicListenerWrapper { socket, config }))
    }
}

impl NetworkListener for QuicListenerWrapper {
    fn accept(
        mut self: Box<QuicListenerWrapper>,
    ) -> NetResult<(Box<dyn NetworkCommunicationChannel>, Option<SocketAddr>)> {
        let mut buf = [0; MAX_DATAGRAM_SIZE];

        self.socket.set_nonblocking(false)?;
        self.socket.set_read_timeout(None)?;
        loop {
            if let Ok((len, from)) = self.socket.recv_from(&mut buf) {
                let buf_slice = &mut buf[..len];
                let hdr = match quiche::Header::from_slice(buf_slice, quiche::MAX_CONN_ID_LEN) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                if hdr.ty != quiche::Type::Initial {
                    continue;
                }

                let to = self.socket.local_addr()?;

                let mut conn = quiche::accept(&hdr.dcid, None, to, from, &mut self.config)?;

                let recv_info = quiche::RecvInfo { to, from };
                let _ = conn.recv(buf_slice, recv_info);
                flush_egress(&self.socket, &mut conn);

                self.socket.set_nonblocking(true)?;

                while !conn.is_established() && !conn.is_closed() {
                    wait_for_data(&self.socket, &mut conn)?;
                    flush_egress(&self.socket, &mut conn);
                }

                return Ok((
                    Box::new(QuicCommunicationChannel {
                        socket: self.socket,
                        conn,
                        stream_id: STREAM_ID,
                    }),
                    Some(to),
                ));
            }
        }
    }
}

impl NetworkCommunicationChannel for QuicCommunicationChannel {
    fn read(&mut self, buffer: &mut [u8]) -> Result<TransferStats, std::io::Error> {
        let mut read = 0;

        while read < buffer.len() {
            match self.conn.stream_recv(self.stream_id, &mut buffer[read..]) {
                Ok((current_read, _fin)) => read += current_read,
                Err(quiche::Error::Done) | Err(quiche::Error::InvalidStreamState(_)) => {
                    flush_egress(&self.socket, &mut self.conn);
                    wait_for_data(&self.socket, &mut self.conn)?;
                }
                Err(e) => return Err(map_quiche_err(e)),
            }
        }

        flush_egress(&self.socket, &mut self.conn);

        let mut stats = TransferStats::empty();
        stats.received(read);
        Ok(stats)
    }

    fn write(
        &mut self,
        buffer: &[u8],
        _: Option<SocketAddr>,
    ) -> Result<TransferStats, std::io::Error> {
        let mut written = 0;
        let stream_id = self.stream_id;

        while written < buffer.len() {
            match self.conn.stream_send(stream_id, &buffer[written..], false) {
                Ok(v) => written += v,
                Err(quiche::Error::InvalidStreamState(_)) | Err(quiche::Error::Done) => {
                    flush_egress(&self.socket, &mut self.conn);
                    wait_for_data(&self.socket, &mut self.conn)?;
                }
                Err(e) => return Err(map_quiche_err(e)),
            }
        }

        flush_egress(&self.socket, &mut self.conn);

        let mut stats = TransferStats::empty();
        stats.sent(written);
        Ok(stats)
    }

    fn sw_read(&mut self, buffer: &mut [u8]) -> Result<TransferStats, std::io::Error> {
        let mut stats = self.read(buffer)?;

        let ack_buf = [1u8];
        let stats_write = self.write(&ack_buf, None)?;
        stats.merge(&stats_write);

        Ok(stats)
    }

    fn sw_write(
        &mut self,
        buffer: &[u8],
        to: Option<SocketAddr>,
    ) -> Result<TransferStats, std::io::Error> {
        let mut stats = self.write(buffer, to)?;

        let mut ack_buf = [0u8; 1];
        let stats_read = self.read(&mut ack_buf)?;
        stats.merge(&stats_read);

        Ok(stats)
    }
}
