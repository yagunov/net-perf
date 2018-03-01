#[macro_use]
extern crate quicli;
use quicli::prelude::*;

use std::path::PathBuf;
use std::net::SocketAddr;


trait Command {
    fn run(&self) -> Result<()>;
}


/// Simple network performance tester
#[derive(Debug, StructOpt)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
enum Cli {
    /// Run in client mode, connect to <host> and transmit <file>
    #[structopt(name="client", alias="c")]
    Client(Client),

    /// Run in server mode (recipient)
    #[structopt(name="server", alias="s", alias="srv")]
    Server(Server),
}

/// Client specific options:
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

impl Command for Client {
    fn run(&self) -> Result<()> {
        println!("Client: {:#?}", self);
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

impl Command for Server {
    fn run(&self) -> Result<()> {
        println!("Server: {:#?}", self);
        Ok(())
    }
}


/// Application's entry point (wrapped for error handling).
main!(|args: Cli| {
    match args {
        Cli::Client(ref client) => client.run()?,
        Cli::Server(ref server) => server.run()?,
    }
});
