use zmq;
use std::{cmp, time};
use std::cell::RefCell;
use utils::*;
use super::*;



pub struct ZmqEngine {
    context: zmq::Context,
    mode: ZmqMode,
}

pub enum ZmqMode {
    PushPull,
    PubSub,
    Pair,
}

struct Client {
    socket: zmq::Socket,
}

struct Server {
    socket: zmq::Socket,
    buf: RefCell<Vec<u8>>,
}

impl ZmqEngine {
    pub fn new(mode: ZmqMode) -> Result<Box<Self>> {
        Ok(Box::new(Self {
            context: zmq::Context::new(),
            mode: mode,
        }))
    }
}

impl Engine for ZmqEngine {
    fn transmitter(&self, destination: &SocketAddr) -> Result<Box<Transmitter>> {
        let protocol = match self.mode {
            ZmqMode::PushPull => zmq::PUSH,
            ZmqMode::PubSub => zmq::PUB,
            ZmqMode::Pair => zmq::PAIR,
        };
        let endpoint = format!("tcp://{}", destination);

        let socket = self.context.socket(protocol)?;
        socket.connect(endpoint.as_str())?;

        Ok(Box::new(Client { socket }))
    }

    fn receiver(&self, port: u16) -> Result<Box<Receiver>> {
        let protocol = match self.mode {
            ZmqMode::PushPull => zmq::PULL,
            ZmqMode::PubSub => zmq::SUB,
            ZmqMode::Pair => zmq::PAIR,
        };
        let endpoint = format!("tcp://*:{}", port);

        let socket = self.context.socket(protocol)?;
        socket.bind(endpoint.as_str())?;
        if protocol == zmq::SUB {
            socket.set_subscribe(b"")?;
        }

        Ok(Box::new(Server {
            socket: socket,
            buf: RefCell::new(Vec::with_capacity(128 * 1024))
        }))
    }
}

impl Transmitter for Client {
    fn transmit(&self, buf: &[u8], chunk_size: usize) -> Result<(PeerInfo, Stats)> {
        let t = time::Instant::now();
        let total = buf.len();
        let mut offset = 0;

        while offset < total {
            let remain = total - offset;
            let chunk = cmp::min(remain, chunk_size);
            self.socket.send(&buf[offset..(offset + chunk)], 0)?;
            offset += chunk;
        }

        Ok((
            PeerInfo {
                src: format!("local"),
                dest: format!("remote"),
            },
            Stats {
                summary: false,
                period: t.elapsed(),
                bytes: total,
                operations: total / chunk_size,
            }))
    }
}


impl Receiver for Server {
    fn receive(&self) -> Result<Option<(PeerInfo, Stats)>> {
        let mut buf = self.buf.borrow_mut();

        let mut total = 0;
        let mut reads = 0;

        let t = time::Instant::now();

        while let Ok(size) = self.socket.recv_into(&mut buf, 0) {
            total += size;
            reads += 1;
            if total > 200_000_000 { break }
        }

        Ok(Some((
            PeerInfo {
                src: format!("local"),
                dest: format!("remote"),
            },
            Stats {
                summary: false,
                period: t.elapsed(),
                bytes: total,
                operations: reads,
            })))
    }
}
