mod args;
mod program;

use std::{
    self, fs,
    io::{self, Result, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use args::GenArgs;
use clap::Parser;
use miow::pipe::{NamedPipe, NamedPipeBuilder};

fn main() -> Result<()> {
    let args = GenArgs::parse();

    match (&args.destination, &args.daemon) {
        (Some(dest), true) => {
            run_daemon(dest, &args)?;
        }
        (Some(dest), false) => {
            let dest = std::path::Path::new(dest);

            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)
                    .unwrap_or_else(|_| panic!("Failed to create directories for {:?}", parent));
            }

            let writer = fs::File::create(dest)
                .unwrap_or_else(|_| panic!("Failed to create file {:?}", dest));

            program::run(args.clone(), Arc::new(Mutex::new(writer)));
        }
        (None, _) => {
            let writer = io::stdout();
            program::run(args.clone(), Arc::new(Mutex::new(writer)));
        }
    };

    Ok(())
}

fn run_daemon(path: &Path, args: &GenArgs) -> Result<()> {
    loop {
        let pipe = DaemonWriter::new(path)?;

        match pipe.connect() {
            Ok(()) => {
                // Handle the connection, e.g., spawn a thread to handle requests
                // For now, we just print a message
                let writer = Arc::new(Mutex::new(pipe));
                program::run(args.clone(), writer.clone());
            }
            Err(e) => {
                eprintln!("Failed to connect to daemon: {}", e);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
}

pub enum DaemonWriter {
    Unix(PathBuf),
    Windows(NamedPipe),
}

impl DaemonWriter {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        if path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Named pipe already exists",
            ));
        }

        #[cfg(unix)]
        {
            todo!("Implement Unix named pipe creation");
        }

        #[cfg(windows)]
        {
            Ok(DaemonWriter::Windows(
                NamedPipeBuilder::new(path)
                    .inbound(false)
                    .outbound(true)
                    .create()?,
            ))
        }
    }

    pub fn connect(&self) -> Result<()> {
        match self {
            DaemonWriter::Unix(path) => {
                todo!("Implement Unix named pipe connection");
            }
            DaemonWriter::Windows(pipe) => pipe.connect(),
        }
    }
}

impl Write for DaemonWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            DaemonWriter::Unix(_) => {
                todo!("Implement Unix named pipe write");
            }
            DaemonWriter::Windows(pipe) => pipe.write(buf),
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        match self {
            DaemonWriter::Unix(_) => {
                todo!("Implement Unix named pipe flush");
            }
            DaemonWriter::Windows(pipe) => pipe.flush(),
        }
    }
}
