mod args;
mod program;

use std::{
    self, fs,
    io::{self, Result, Write},
};

use args::GenArgs;
use clap::Parser;

fn main() -> Result<()> {
    let args = GenArgs::parse();

    let output = program::run(args.clone());

    match args.destination {
        Some(d) => fs::write(d.clone(), output)
            .unwrap_or_else(|_| panic!("Failed to write to file {:?}", d)),
        None => io::stdout()
            .write_all(output.as_bytes())
            .expect("Failed to write to stdout"),
    }

    Ok(())
}
