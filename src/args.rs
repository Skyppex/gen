use std::str::FromStr;

use clap::{error::ErrorKind, ArgGroup, Error, Parser, Subcommand};

/// Write a concise description of the command here.
#[derive(Debug, Clone, Parser)]
#[command(version, author, about)]
#[command(group=ArgGroup::new("log").args(["verbose", "quiet"]).multiple(false))]
pub struct GenArgs {
    // /// The source file to read from. If not provided, read from stdin.
    // #[arg(short, long)]
    // pub source: Option<String>,
    /// The destination file to write to. If not provided, write to stdout.
    #[arg(short, long)]
    pub destination: Option<String>,

    /// Enable verbose logging.
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress all informational output.
    /// Errors will still be printed to stderr.
    #[arg(short, long)]
    pub quiet: bool,

    #[command(subcommand)]
    pub commands: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Generate a random integer.
    /// Use conventional range notation (e.g. 1..100).
    /// The range is inclusive.
    /// If no range is specified, the default is 0..100.
    Int { range: Option<IntRange> },

    /// Generate a random floating-point number.
    /// Use conventional range notation (e.g. 1.0..100.0).
    /// The range is inclusive.
    /// If no range is specified, the default is 0.0..1.0.
    Float { range: Option<FloatRange> },
    /// Generate a random UUID.
    /// Optionally specify the version.
    /// If not specified, version 4 is used.
    Uuid {
        /// The version of the UUID to generate.
        /// If not specified, version 4 is used.
        /// Possible values: empty, 4, max.
        version: Option<UuidVersion>,
    },
    /// Generate a random URL.
    /// Optionally specify the length of the generated strings
    /// and the number of path segments.
    Url {
        /// The length of the generated strings.
        length: Option<usize>,

        /// The number of path segments.
        #[arg(short, long)]
        path: Option<Option<u8>>,

        /// Include a query string.
        #[arg(short, long)]
        query: bool,
    },
}

#[derive(Debug, Clone, Parser)]
pub struct IntRange {
    pub min: i32,
    pub max: i32,
}

impl FromStr for IntRange {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split("..").collect();

        if parts.len() != 2 {
            return Err(Error::new(ErrorKind::ValueValidation));
        }

        let min = parts[0]
            .parse()
            .map_err(|_| Error::new(clap::error::ErrorKind::ValueValidation))?;
        let max = parts[1]
            .parse()
            .map_err(|_| Error::new(clap::error::ErrorKind::ValueValidation))?;

        Ok(IntRange { min, max })
    }
}

#[derive(Debug, Clone, Parser)]
pub struct FloatRange {
    pub min: f32,
    pub max: f32,
}

impl FromStr for FloatRange {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split("..").collect();

        if parts.len() != 2 {
            return Err(Error::new(clap::error::ErrorKind::ValueValidation));
        }

        let min = parts[0]
            .parse()
            .map_err(|_| Error::new(clap::error::ErrorKind::ValueValidation))?;
        let max = parts[1]
            .parse()
            .map_err(|_| Error::new(clap::error::ErrorKind::ValueValidation))?;

        Ok(FloatRange { min, max })
    }
}

#[derive(Debug, Clone, Parser)]
pub enum UuidVersion {
    Empty,
    V4,
    Max,
}

impl FromStr for UuidVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "empty" => Ok(UuidVersion::Empty),
            "4" | "v4" | "ver4" | "version4" => Ok(UuidVersion::V4),
            "max" => Ok(UuidVersion::Max),
            _ => Err(Error::new(clap::error::ErrorKind::ValueValidation)),
        }
    }
}
