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
    /// Fatal errors will still be printed to stderr.
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
    /// default: 0..100.
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
        #[arg(short, long)]
        length: Option<usize>,

        /// The number of path segments.
        #[arg(short, long)]
        path: Option<Option<u8>>,

        /// Include a query string.
        #[arg(short, long)]
        query: bool,
    },

    Ascii {
        /// Choose a specific character set.
        #[arg(short, long)]
        charset: Option<String>,

        /// Choose a specific character set.
        #[arg(long, num_args = 1..)]
        charset_ranges: Option<Vec<IntRange>>,

        /// Exclude a specific character set.
        #[arg(short, long)]
        exclude: Option<String>,

        /// Exclude a set of character codes.
        #[arg(long, num_args = 1..)]
        exclude_codes: Option<Vec<u8>>,

        /// Size of the output.
        size: ByteSize,
    },

    Unicode {
        encoding: UnicodeEncoding,

        /// Choose a specific character set.
        #[arg(short, long)]
        charset: Option<String>,

        /// Exclude a specific character set.
        #[arg(short, long)]
        exclude: Option<String>,

        /// Size of the output.
        size: ByteSize,
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

#[derive(Debug, Clone, Parser)]
pub struct ByteSize {
    pub value: usize,
    pub unit: ByteUnit,
}

#[derive(Debug, Clone, Parser)]
pub enum ByteUnit {
    B,
    KB,
    MB,
    GB,
}

impl ByteSize {
    pub fn to_bytes(&self) -> usize {
        match self.unit {
            ByteUnit::B => self.value,
            ByteUnit::KB => self.value * 1024,
            ByteUnit::MB => self.value * 1024 * 1024,
            ByteUnit::GB => self.value * 1024 * 1024 * 1024,
        }
    }
}

impl FromStr for ByteSize {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut string_value = String::new();

        for c in s.chars() {
            if c.is_ascii_digit() {
                string_value.push(c);
            } else {
                break;
            }
        }

        Ok(ByteSize {
            value: string_value
                .parse()
                .map_err(|_| Error::new(clap::error::ErrorKind::ValueValidation))?,
            unit: s[string_value.len()..].parse()?,
        })
    }
}

impl FromStr for ByteUnit {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "B" | "b" => Ok(ByteUnit::B),
            "KB" | "kb" => Ok(ByteUnit::KB),
            "MB" | "mb" => Ok(ByteUnit::MB),
            "GB" | "gb" => Ok(ByteUnit::GB),
            _ => Err(Error::new(clap::error::ErrorKind::ValueValidation)),
        }
    }
}

#[derive(Debug, Clone, Parser)]
pub enum UnicodeEncoding {
    Utf8,
    Utf16,
    Utf32,
}

impl FromStr for UnicodeEncoding {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "utf_8" | "utf8" | "8" => Ok(UnicodeEncoding::Utf8),
            "utf_16" | "utf16" | "16" => Ok(UnicodeEncoding::Utf16),
            "utf_32" | "utf32" | "32" => Ok(UnicodeEncoding::Utf32),
            _ => Err(Error::new(clap::error::ErrorKind::ValueValidation)),
        }
    }
}
