use std::net::SocketAddr;
use std::collections::HashMap;
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
    fn transmit(&self, buf: &[u8], chunk_size: usize) -> Result<(PeerInfo, Stats)>;

    /// Execute transmitter
    fn run(&self, buf: &[u8], chunk_size: usize, repeat: usize) -> Result<()> {
        let mut avg = AvgStats::new();

        for i in 1..repeat+1 {
            let (peers, stats) = self.transmit(buf, chunk_size)?;

            println!(
                "[{}] {} -> {}: {:.3} Gbps / {:.2} Mops",
                i, peers.src, peers.dest, stats.gbps(), stats.mops()
            );

            avg.update(peers, &stats);
        }

        for (peers, avg_gbps, avg_mops) in avg.iter() {
            println!("[SUM] {} -> {}: {:.3} Gbps / {:.2} Mops", peers.src, peers.dest, avg_gbps, avg_mops);
        }

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
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct PeerInfo {
    src: String,                // TODO: Use SocketAddr instead
    dest: String,
}

/// Statistics report
pub struct Stats {
    summary: bool,
    period: time::Duration,
    bytes: usize,
    operations: usize,
}

impl Stats {
    fn zero() -> Self {
        Self {
            summary: false,
            period: time::Duration::from_secs(0),
            bytes: 0,
            operations: 0,
        }
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

/// Average statistics per connection
struct AvgStats {
    table: HashMap<PeerInfo, Stats>,
}

impl AvgStats {
    fn new() -> Self {
        Self { table: HashMap::new() }
    }

    fn update(&mut self, peers: PeerInfo, stats: &Stats) {
        let entry = self.table.entry(peers).or_insert(Stats::zero());
        entry.period += stats.period;
        entry.bytes += stats.bytes;
        entry.operations += stats.operations;
    }

    fn iter<'a>(&'a self) -> Box<Iterator<Item=(&PeerInfo, f64, f64)> + 'a> {
        Box::new(
            self.table
                .iter()
                .map(|(peers, stats)| {
                    (peers, stats.gbps(), stats.mops())
                })
        )
    }
}


mod tcp;
mod zmq;

pub use self::tcp::TCPEngine;
pub use self::zmq::ZMQEngine;
