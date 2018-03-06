use std::time;
use std::cell::RefCell;
use std::cmp;
use std::sync::mpsc;
use std::thread;
use utils::*;
use super::*;

use std::net::{TcpStream, TcpListener, Shutdown};
use std::io::{Read, Write};


pub struct TCPEngine {}

struct Client {
    stream: RefCell<TcpStream>,
}

struct Server {
    reports: mpsc::Receiver<Option<(PeerInfo, Stats)>>,
}


impl TCPEngine {
    pub fn new() -> Result<Box<Self>> {
        Ok(Box::new(Self {}))
    }
}

impl Engine for TCPEngine {
    fn transmitter(&self, destination: &SocketAddr) -> Result<Box<Transmitter>> {
        Ok(Box::new(Client {
            stream: RefCell::new(TcpStream::connect(destination)?)
        }))
    }

    fn receiver(&self, port: u16) -> Result<Box<Receiver>> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            // Process incoming connections in background threads
            for conn in listener.incoming() {
                conn.map(|stream| {
                    let tx = tx.clone();
                    thread::spawn(move || handle_client(stream, tx))
                }).unwrap();
            }
            // Send end of stream marker
            tx.send(None).unwrap();
        });

        Ok(Box::new(Server {
            reports: rx,
        }))
    }
}

impl Transmitter for Client {
    fn transmit(&self, buf: &[u8], chunk_size: usize) -> Result<(PeerInfo, Stats)> {
        let t = time::Instant::now();
        let mut stream = self.stream.borrow_mut();
        let total = buf.len();
        let mut offset = 0;

        while offset < total {
            let remain = total - offset;
            let chunk = cmp::min(remain, chunk_size);
            offset += stream.write(&buf[offset..(offset + chunk)])?;
        }

        Ok((
            PeerInfo {
                src: format!("{}", stream.local_addr()?),
                dest: format!("{}", stream.peer_addr()?),
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
        Ok(self.reports.recv()?)
    }
}


fn handle_client(mut stream: TcpStream, tx: mpsc::Sender<Option<(PeerInfo, Stats)>>) -> Result<()> {
    let peers = PeerInfo {
        src: format!("{}", stream.peer_addr()?),
        dest: format!("{}", stream.local_addr()?),
    };

    let mut buf = [0u8; 128 * 1024];

    let mut t = time::Instant::now();
    let mut total = 0;
    let mut nb_reads = 0;

    while let Ok(size) = stream.read(&mut buf) {
        if size == 0 {
            tx.send(Some((
                peers.clone(),
                Stats {
                    summary: true,
                    period: t.elapsed(),
                    bytes: total,
                    operations: nb_reads,
                }
            )))?;

            stream.shutdown(Shutdown::Both)?;
            return Ok(());
        }

        total += size;

        // report approximated speed every 500MB
        if total >= 500_000_000 {
            tx.send(Some((
                peers.clone(),
                Stats {
                    summary: false,
                    period: t.elapsed(),
                    bytes: total,
                    operations: nb_reads,
                }
            )))?;

            // reset counters
            t = time::Instant::now();
            total = 0;
            nb_reads = 0;
        }
    }

    Ok(())
}
