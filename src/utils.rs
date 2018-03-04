pub use quicli::prelude::*;
use zmq;

pub use std::fs::File;
pub use std::io::Read;
pub use std::net::SocketAddr;
pub use std::path::{Path, PathBuf};
pub use std::time;
pub use std::thread;


#[allow(unused_macros)]
macro_rules! bail {
    ($e:expr) => {
        return Err(format_err!($e));
    };
    ($fmt:expr, $($arg:tt)+) => {
        return Err(format_err!($fmt, $($arg)+).into());
    };
}


/// Convert duration into seconds (floating).
pub trait Seconds { fn seconds(&self) -> f64; }
impl Seconds for time::Duration {
    fn seconds(&self) -> f64 {
        self.as_secs() as f64 + (self.subsec_nanos() as f64 / 1e9)
    }
}

/// Read files content into memory
pub fn load_file<P: AsRef<Path>>(path: P) -> Result<(Vec<u8>, time::Duration)> {
    let mut content = vec![];
    let mut f = File::open(path)?;
    let t = time::Instant::now();
    f.read_to_end(&mut content)?;
    Ok((content, t.elapsed()))
}

/// Calculate transmission speed.
pub fn speed_gbps(payload: usize, d: time::Duration) -> f64 {
    let bits = payload * 8;
    (bits as f64 / d.seconds()) / 1073741830f64
}

pub trait Command {
    fn run(&self, engine: Box<Engine>) -> Result<()>;
}


pub trait Transiver: Send {
    fn peers(&self) -> (String, String);
    fn transmit(&self, buf: &[u8], chunk_size: usize) -> Result<time::Duration>;

    fn run(&self, buf: &[u8], chunk_size: usize, repeat: usize) -> Result<()> {
        let size = buf.len();
        let (from, to) = self.peers();

        for i in 0..repeat {
            let dt = self.transmit(buf, chunk_size)?;
            println!("[{}] {} -> {}: {} Gbps / {} ops/sec", i+1, from, to, speed_gbps(size, dt), size / chunk_size);
        }

        // println!("[SUM] {} -> {}: {} Gbps / {} ops/sec", i+1, from, to, speed_gbps(size, dt));

        Ok(())
    }
}

pub trait Receiver: Send {
    fn run(&self) -> Result<()>;
}

pub trait Engine {
    fn transmitter(&self, destination: &SocketAddr) -> Result<Box<Transiver>>;
    fn receiver(&self, port: u16) -> Result<Box<Receiver>>;
}

pub struct TCPEngine {
    base_port: u16,
}

impl TCPEngine {
    pub fn new() -> Result<Box<Self>> {
        Ok(Box::new(Self {
            base_port: 10000,
        }))
    }
}

impl Engine for TCPEngine {
    fn transmitter(&self, destination: &SocketAddr) -> Result<Box<Transiver>> {
        Ok(Box::new(TCPWriter {}))
    }

    fn receiver(&self, port: u16) -> Result<Box<Receiver>> {
        Ok(Box::new(TCPReader {}))
    }
}

pub struct TCPWriter { }

impl Transiver for TCPWriter {
    fn peers(&self) -> (String, String) {
        ("local".to_owned(), "remote".to_owned())
    }

    fn transmit(&self, buf: &[u8], chunk_size: usize) -> Result<time::Duration> {
        let t = time::Instant::now();
        Ok(t.elapsed())
    }
}

pub struct TCPReader { }

impl Receiver for TCPReader {
    fn run(&self) -> Result<()> {
        Ok(())
    }
}


pub struct ZMQEngine {
    context: zmq::Context,
    socket_type: zmq::SocketType,
}

impl ZMQEngine {
    pub fn new(socket_type: zmq::SocketType) -> Result<Box<Self>> {
        Ok(Box::new(Self {
            context: zmq::Context::new(),
            socket_type: socket_type,
        }))
    }
}

impl Engine for ZMQEngine {
    fn transmitter(&self, destination: &SocketAddr) -> Result<Box<Transiver>> {
        Ok(Box::new(TCPWriter {}))
    }

    fn receiver(&self, port: u16) -> Result<Box<Receiver>> {
        Ok(Box::new(TCPReader {}))
    }
}
