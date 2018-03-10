#[macro_use]
extern crate quicli;
extern crate zmq;


/// Measure execution time of expression or code block.
macro_rules! timeit {
    ($e:expr) => ({
        let t = time::Instant::now();
        let res = $e;
        (t.elapsed(), res)
    })
}

mod utils;
mod engines;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use utils::*;
use engines::*;


/// Simple network performance tester
#[derive(Debug, StructOpt)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
struct Cli {
    /// Transport protocol to use
    #[structopt(long="proto", short="P", default_value="tcp")]
    proto: String,

    #[structopt(subcommand)]
    mode: Mode,
}

/// Operating modes
#[derive(Debug, StructOpt)]
enum Mode {
    /// Run in client mode, connect to <host> and transmit <file>
    #[structopt(name="client", alias="c")]
    Client(Client),

    /// Run in server mode (recipient)
    #[structopt(name="server", alias="s", alias="srv")]
    Server(Server),
}

/// Client specific options
#[derive(Debug, StructOpt)]
struct Client {
    // First positional argument:
    /// File to transmit
    #[structopt(parse(from_os_str))]
    file: PathBuf,

    // Second positional argument (can be a list):
    /// Remove host and socket addresses (new thread will be started for each remote socket)
    remote: Vec<SocketAddr>,

    /// Maximum size of transmission chunk
    #[structopt(long="chunk", short="C", default_value="512")]
    chunk_size: usize,

    /// Count (example argument)
    #[structopt(long="loop", short="l", default_value="1")]
    repeat: usize,
}

impl Client {
    fn run(&self, engine: Box<Engine>) -> Result<()> {
        if self.remote.len() == 0 {
            bail!("Need at least one destination address");
        }

        println!("Client: {:#?}", self);

        // Read payload from user specified file
        let (load_time, payload) = timeit!(load_file(&self.file)?);
        let load_time = load_time.seconds();
        let size_mb = payload.len() as f64 / 1048576f64;
        let speed_mb = size_mb / load_time;
        println!("Payload: {:.2} MB ({}) loaded in {} = {:.2} MB/s", size_mb, payload.len(), load_time, speed_mb);

        let payload = Arc::new(payload);
        let mut handlers = vec![];

        for dest in self.remote.iter() {
            let tx = engine.transmitter(dest)?;
            let buf = payload.clone();
            let chunk_size = self.chunk_size;
            let repeat = self.repeat;
            handlers.push(thread::spawn(move || { tx.run(&buf, chunk_size, repeat) }));
        }

        for handler in handlers.into_iter() {
            handler.join().unwrap()?;
        }

        Ok(())
    }
}


/// Server specific options:
#[derive(Debug, StructOpt)]
struct Server {
    /// Base port number (will be incremented for each server thread)
    #[structopt(long="port", short="p", default_value="10000")]
    port: u16,

    /// Number of threads to run
    #[structopt(long="threads", short="t", default_value="1")]
    threads: u8,
}

impl Server {
    fn run(&self, engine: Box<Engine>) -> Result<()> {
        println!("Server: {:#?}", self);

        let mut handlers = vec![];

        for i in 0..self.threads {
            let rx = engine.receiver(self.port + i as u16)?;
            let handler = thread::spawn(move || { rx.run() });
            handlers.push(handler);
        }

        for handler in handlers.into_iter() {
            handler.join().unwrap()?;
        }

        Ok(())
    }
}


/// Application's entry point (wrapped for error handling).
main!(|args: Cli| {
    let engine: Box<Engine> = match args.proto.as_ref() {
        "tcp" => TcpEngine::new()?,
        "zmq" |
        "zmq:pushpull" => ZmqEngine::new(ZmqMode::PushPull)?,
        "zmq:pubsub" => ZmqEngine::new(ZmqMode::PubSub)?,
        "zmq:pair" => ZmqEngine::new(ZmqMode::Pair)?,
        _ => bail!("Unsupported protocol"),
    };

    match args.mode {
        Mode::Client(ref client) => client.run(engine)?,
        Mode::Server(ref server) => server.run(engine)?,
    }
});
