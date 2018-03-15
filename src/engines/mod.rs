use std::net::SocketAddr;
use utils::*;


/// Network protocol driver
pub trait Engine {
    /// Create new transmitter
    fn transmitter(&self, destination: &SocketAddr) -> Result<Box<Transmitter>>;

    /// Create new receiver
    fn receiver(&self, port: u16) -> Result<Box<Receiver>>;
}

/// Transmitter (client) interface
pub trait Transmitter: Send {
    /// Transmit payload by chunks
    fn transmit(&self, buf: &[u8], chunk_size: usize) -> Result<Stats>;

    /// Describe connection endpoints with human-readable tags
    fn peers(&self) -> Result<PeerInfo> {
        Ok(PeerInfo::new("local", "remote"))
    }

    /// Execute transmitter
    fn run(&self, buf: &[u8], chunk_size: usize, repeat: usize) -> Result<()> {
        let peers = self.peers()?;

        let mut cumulative = Stats::zero(true);
        let mut i = 1;

        while i != repeat {     // repeat == 0 => infinite loop
            let stats = self.transmit(buf, chunk_size)?;
            cumulative.update(&stats);

            println!(
                "[{}] {} -> {}: {:.3} Gbps / {:.2} Mops",
                i, peers.src, peers.dest, stats.gbps(), stats.mops()
            );

            i += 1;
        }

        println!(
            "[SUM] {} -> {}: {:.3} Gbps / {:.2} Mops",
            peers.src, peers.dest, cumulative.gbps(), cumulative.mops()
        );

        Ok(())
    }
}

/// Receiver (server) interface.
pub trait Receiver: Send {
    /// Report receive statistics.
    fn receive(&self) -> Result<Option<(PeerInfo, Stats)>>;

    fn run(&self) -> Result<()> {
        while let Some((peers, stats)) = self.receive()? {
            println!(
                "[{}] {} -> {}: {:.3} Gbps / {:.2} Mops",
                if stats.summary { "SUM" } else { "-" },
                peers.src, peers.dest, stats.gbps(), stats.mops()
            );
        }
        Ok(())
    }
}


/// Connection endpoints
#[derive(Clone)]
pub struct PeerInfo {
    src: String,
    dest: String,
}

impl PeerInfo {
    pub fn new(src: &str, dest: &str) -> PeerInfo {
        PeerInfo { src: src.to_owned(), dest: dest.to_owned() }
    }
}

/// Statistics report
pub struct Stats {
    summary: bool,
    period: time::Duration,
    bytes: usize,
    operations: usize,
}

impl Stats {
    fn zero(summary: bool) -> Self {
        Self {
            summary,
            period: time::Duration::from_secs(0),
            bytes: 0,
            operations: 0,
        }
    }

    fn summary(period: time::Duration, bytes: usize, operations: usize) -> Self {
        Self { summary: true, period, bytes, operations }
    }

    fn partial(period: time::Duration, bytes: usize, operations: usize) -> Self {
        Self { summary: false, period, bytes, operations }
    }

    fn update(&mut self, partial: &Stats) {
        self.period += partial.period;
        self.bytes += partial.bytes;
        self.operations += partial.operations;
    }

    /// Calculate transmission speed in Gbps
    fn gbps(&self) -> f64 {
        let bits = self.bytes * 8;
        (bits as f64 / self.period.seconds()) / 1073741824f64
    }

    /// Mega-operations per second
    fn mops(&self) -> f64 {
        (self.operations as f64 / self.period.seconds()) / 1e6f64
    }
}

/// Read at most this number of bytes at a time
pub const RECV_BUF_SIZE: usize = 128 * 1024;

/// Report receiver statistics after that many bytes
pub const RECV_REPORT_INTERVAL: usize = 200 * 1024 * 1024;


mod tcp;
mod zmq;

pub use self::tcp::*;
pub use self::zmq::*;
