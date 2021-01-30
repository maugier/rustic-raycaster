use anyhow::{Result,Context};
use rustic_raycaster::loader::Map;
use std::io::BufReader;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: render <CUB FILE> <OUTPUT FILE>");
        std::process::exit(1);
    }

    let fh = std::fs::File::open(&args[1])?;
    let buf = BufReader::new(fh);

    let map = Map::load(buf).context("Failed to load map")?;

    Ok(())

}