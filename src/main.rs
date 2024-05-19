mod args;
mod program;
mod utils;

use std::{self, fs, io::{self, Read, Result, Write}, process};

use args::Args;
use clap::Parser;
use utils::convert_to_path;

fn main() -> Result<()> {
    let args = Args::parse();
    
    let source = convert_to_path(&args.source)?;
    let destination = convert_to_path(&args.destination)?;

    let input = match source {
        Some(path) => fs::read_to_string(path)?,
        None => {
            if atty::is(atty::Stream::Stdin) {
                eprintln!("%PROGRAM% -> No input given");
                process::exit(1);
            }

            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer
        }
    };

    let output = program::run(input, args);

    match destination {
        Some(d) => {
            match fs::write(d.clone(), output) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Failed to write to file: {:?}", d);
                    eprintln!("{}", e);
                    process::exit(1);
                }
            }
        },
        None => {
            match io::stdout().write_all(output.as_bytes()) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Failed to write to stdout");
                    eprintln!("{}", e);
                    process::exit(1);
                }
            }
        }
    }

    Ok(())
}
