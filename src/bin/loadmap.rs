
use rustic_raycaster::loader::Map;
use anyhow::{Context, Result};
use std::io::BufReader;

fn main() -> Result<()> {

    let args: Vec<_> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: loadmap <MAP FILE>");
        std::process::exit(1);
    }

    let fh = std::fs::File::open(&args[1])?;
    let buf = BufReader::new(fh);

    let map = Map::load(buf).context("Failed to load map")?;

    println!("{:?}", map);

    Ok(())

}
