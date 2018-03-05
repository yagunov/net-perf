use std::time;
use utils::*;
use super::*;

use std::net::{TcpStream, TcpListener, Shutdown};


pub struct TCPEngine {}

struct Client {}
struct Server {}


impl TCPEngine {
    pub fn new() -> Result<Box<Self>> {
        Ok(Box::new(Self {}))
    }
}

impl Engine for TCPEngine {
    fn transmitter(&self, destination: &SocketAddr) -> Result<Box<Transmitter>> {
        Ok(Box::new(Client {}))
    }

    fn receiver(&self, port: u16) -> Result<Box<Receiver>> {
        Ok(Box::new(Server {}))
    }
}

impl Transmitter for Client {
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


impl Receiver for Server {
    fn receive(&self) -> Result<Option<(PeerInfo, Stats)>> {
        Ok(None)
    }
}
