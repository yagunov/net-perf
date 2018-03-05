use zmq;
use std::time;
use utils::*;
use super::*;



pub struct ZMQEngine {
    context: zmq::Context,
    socket_type: zmq::SocketType,
}

struct Socket {}

impl ZMQEngine {
    pub fn new(socket_type: zmq::SocketType) -> Result<Box<Self>> {
        Ok(Box::new(Self {
            context: zmq::Context::new(),
            socket_type: socket_type,
        }))
    }
}

impl Engine for ZMQEngine {
    fn transmitter(&self, destination: &SocketAddr) -> Result<Box<Transmitter>> {
        Ok(Box::new(Socket {}))
    }

    fn receiver(&self, port: u16) -> Result<Box<Receiver>> {
        Ok(Box::new(Socket {}))
    }
}

impl Transmitter for Socket {
    fn transmit(&self, buf: &[u8], chunk_size: usize) -> Result<(PeerInfo, Stats)> {
        let now = time::Instant::now();
        Ok((
            PeerInfo {
                src: "local".to_owned(),
                dest: "remote".to_owned()
            },
            Stats {
                period: now.elapsed(),
                bytes: buf.len(),
                operations: buf.len() / chunk_size,
            }))
    }
}


impl Receiver for Socket {
    fn receive(&self) -> Result<Option<(PeerInfo, Stats)>> {
        Ok(None)
    }
}
