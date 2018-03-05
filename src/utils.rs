pub use quicli::prelude::*;
pub use std::time;

use std::fs::File;
use std::io::Read;
use std::path::Path;


/// Convert duration into seconds (floating).
pub trait Seconds { fn seconds(&self) -> f64; }
impl Seconds for time::Duration {
    fn seconds(&self) -> f64 {
        self.as_secs() as f64 + (self.subsec_nanos() as f64 / 1e9)
    }
}

/// Read files content into memory
pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let mut content = vec![];
    let mut f = File::open(path)?;
    f.read_to_end(&mut content)?;
    Ok(content)
}
