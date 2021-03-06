use std::cell::RefCell;
use std::io::{Read, Write};
use std::net::{TcpStream, TcpListener, Shutdown};
use std::sync::mpsc;
use std::{cmp, time, thread};
use utils::*;
use super::*;


pub struct TcpEngine {}

struct Client {
    stream: RefCell<TcpStream>,
}

struct Server {
    reports: mpsc::Receiver<Option<(PeerInfo, Stats)>>,
}


impl TcpEngine {
    pub fn new() -> Result<Box<Self>> {
        Ok(Box::new(Self {}))
    }
}

impl Engine for TcpEngine {
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

        Ok(Box::new(Server { reports: rx }))}
}

impl Transmitter for Client {
    fn transmit(&self, buf: &[u8], chunk_size: usize) -> Result<Stats> {
        let t = time::Instant::now();
        let mut stream = self.stream.borrow_mut();
        let total = buf.len();
        let mut offset = 0;

        while offset < total {
            let remain = total - offset;
            let chunk = cmp::min(remain, chunk_size);
            offset += stream.write(&buf[offset..(offset + chunk)])?;
        }

        Ok(Stats::partial(t.elapsed(), total, total / chunk_size))
    }
}

impl Receiver for Server {
    fn receive(&self) -> Result<Option<(PeerInfo, Stats)>> {
        Ok(self.reports.recv()?)
    }
}


fn handle_client(mut stream: TcpStream, tx: mpsc::Sender<Option<(PeerInfo, Stats)>>) -> Result<()> {
    // Disconnect after 1 second of silence
    stream.set_read_timeout(Some(time::Duration::from_secs(1)))?;

    let peers = PeerInfo {
        src: format!("{}", stream.peer_addr()?),
        dest: format!("{}", stream.local_addr()?),
    };

    let mut buf = [0u8; RECV_BUF_SIZE];

    let mut total = 0;
    let mut prev_total = 0;
    let mut reads = 0;
    let mut prev_reads = 0;

    let start = time::Instant::now();
    let mut t = start;

    while let Ok(size) = stream.read(&mut buf) {
        total += size;
        reads += 1;

        if size == 0 {
            // Report summary statistics and terminate socket
            tx.send(Some((peers.clone(), Stats::summary(start.elapsed(), total, reads))))?;
            stream.shutdown(Shutdown::Both)?;
            return Ok(());
        }

        let diff = total - prev_total;
        if diff >= RECV_REPORT_INTERVAL {
            // Report partial statistics
            tx.send(Some((peers.clone(), Stats::partial(t.elapsed(), diff, reads - prev_reads))))?;

            // reset counters
            t = time::Instant::now();
            prev_total = total;
            prev_reads = reads;
        }
    }

    Ok(())
}
