mod args;
mod program;

use std::{
    self, fs,
    io::{self, Result},
};

use args::GenArgs;
use clap::Parser;

fn main() -> Result<()> {
    let args = GenArgs::parse();

    match &args.destination {
        Some(dest) => {
            let writer = fs::File::create(dest.clone())
                .unwrap_or_else(|_| panic!("Failed to create file {:?}", dest));
            program::run(args.clone(), writer);
        }
        None => {
            let writer = io::stdout();
            program::run(args.clone(), writer);
        }
    };

    Ok(())
}
