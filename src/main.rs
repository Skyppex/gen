mod args;
mod program;

use std::{
    self, fs,
    io::{self, Result, Write},
    process,
};

use args::GenArgs;
use clap::Parser;

fn main() -> Result<()> {
    let args = GenArgs::parse();

    let output = program::run(args.clone());

    match args.destination {
        Some(d) => match fs::write(d.clone(), output) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Failed to write to file: {:?}", d);
                eprintln!("{}", e);
                process::exit(1);
            }
        },
        None => match io::stdout().write_all(output.as_bytes()) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Failed to write to stdout");
                eprintln!("{}", e);
                process::exit(1);
            }
        },
    }

    Ok(())
}
