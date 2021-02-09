use anyhow::{Result,Context};
use rustic_raycaster::{geometry::Vector, loader::Map, render::Render};
use std::{io::BufReader, time::Instant};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: render <CUB FILE> <OUTPUT FILE>");
        std::process::exit(1);
    }

    let fh = std::fs::File::open(&args[1])?;
    let buf = BufReader::new(fh);

    let t0 = Instant::now();

    let map = Map::load(buf).context("Failed to load map")?;
    let t1 = Instant::now();

    let mut r = Render::spawn(&map);
    r.cam = Vector::angle(265.0f64.to_radians());
    r.render(&map);
    let t2 = Instant::now();

    eprintln!("Loaded in {:?}, rendered in {:?}", t1-t0, t2-t1);

    r.buffer.save(&args[2])?;

    Ok(())

}