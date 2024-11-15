use std::{fmt::Display, num::NonZeroUsize, str::FromStr};

use clap::{error::ErrorKind, ArgGroup, Error, Parser, Subcommand};

/// Write a concise description of the command here.
#[derive(Debug, Clone, Parser)]
#[command(version, author, about)]
// Only for the ascii subcommand. It doesn't work when i but this line on the enum variant itself
pub struct GenArgs {
    /// The destination file to write to. If not provided, write to stdout.
    #[arg(short, long)]
    pub destination: Option<String>,

    #[command(subcommand)]
    pub commands: Command,
}

#[derive(Subcommand, Debug, Clone)]
#[command(verbatim_doc_comment)]
pub enum Command {
    /// Generate a random integer.
    /// Use conventional range notation (e.g. 1..100).
    /// The range is inclusive.
    /// Default: 0..100.
    #[command(verbatim_doc_comment)]
    Int { range: Option<IntRange> },

    /// Generate a random floating-point number.
    /// Use conventional range notation (e.g. 1.0..100.0).
    /// The range is inclusive.
    /// Default: 0.0..1.0.
    #[command(verbatim_doc_comment)]
    Float { range: Option<FloatRange> },

    /// Generate a random UUID.
    /// Optionally specify the version.
    /// Default: v4.
    #[command(verbatim_doc_comment)]
    Uuid {
        /// The version of the UUID to generate.
        /// Possible values: empty, v4, max.
        /// Default: v4.
        #[arg(verbatim_doc_comment)]
        version: Option<UuidVersion>,
    },

    /// Generate a random URL.
    #[command(verbatim_doc_comment)]
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

    /// Generate a random ASCII string.
    /// Warning: This command may generate non-printable characters and control characters.
    /// Warning: Your terminal emulator might have trouble rendering large output strings.
    ///   If you are trying to generate a lot of data,
    ///   consider using the --destination flag to write to a file.
    #[command(verbatim_doc_comment)]
    #[command(group=ArgGroup::new("chars").args(&["charset", "printable_only"]).multiple(false))]
    Ascii {
        /// Size of the output. Format: <value><unit>.
        /// Possible units: B, KB, MB, GB, KiB, MiB, GiB.
        size: ByteSize,

        /// Choose a specific character set.
        #[arg(short, long)]
        charset: Option<String>,

        /// Only include printable characters.
        #[arg(long)]
        printable_only: bool,

        /// Exclude a specific character set.
        #[arg(short, long)]
        exclude: Option<String>,

        /// Exclude a set of character codes.
        #[arg(long, num_args = 1..)]
        exclude_codes: Option<Vec<u8>>,

        /// The number of threads to use.
        #[arg(short, long)]
        threads: Option<NonZeroUsize>,

        /// The buffer size to use per thread.
        /// The maximum memory allocation will be threads * buf-size.
        /// Warning: The smaller the buffer size,
        /// the slower the generation will be due to more frequent writes.
        #[arg(short, long, verbatim_doc_comment)]
        buf_size: Option<ByteSize>,

        /// Show a progress bar.
        #[arg(short, long, default_value = "false")]
        progress: bool,
    },

    /// Generate a random Unicode string.
    /// Warning: This command may generate non-printable characters and control characters.
    /// Warning: Your terminal emulator might have trouble rendering large output strings.
    #[command(verbatim_doc_comment)]
    Unicode {
        /// Size of the output. Format: <value><unit>.
        /// Possible units: B, KB, MB, GB, KiB, MiB, GiB.
        size: ByteSize,

        /// Choose a specific encoding. Possible values: utf8, utf16, utf32.
        encoding: UnicodeEncoding,

        /// Choose a specific character set.
        #[arg(short, long)]
        charset: Option<String>,

        /// Exclude a specific character set.
        #[arg(short, long)]
        exclude: Option<String>,

        /// The number of threads to use.
        #[arg(short, long)]
        threads: Option<NonZeroUsize>,

        /// The buffer size to use per thread.
        /// The maximum memory allocation will be threads * buf-size.
        /// Warning: The smaller the buffer size,
        /// the slower the generation will be due to more frequent writes.
        #[arg(short, long, verbatim_doc_comment)]
        buf_size: Option<ByteSize>,

        /// Show a progress bar.
        #[arg(short, long, default_value = "false")]
        progress: bool,
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
    KiB,
    MB,
    MiB,
    GB,
    GiB,
}

impl ByteSize {
    pub fn to_bytes(&self) -> usize {
        match self.unit {
            ByteUnit::B => self.value,
            ByteUnit::KB => self.value * 1000,
            ByteUnit::KiB => self.value * 1024,
            ByteUnit::MB => self.value * 1000 * 1000,
            ByteUnit::MiB => self.value * 1024 * 1024,
            ByteUnit::GB => self.value * 1000 * 1000 * 1000,
            ByteUnit::GiB => self.value * 1024 * 1024 * 1024,
        }
    }

    pub fn is_decimal_unit(&self) -> bool {
        match self.unit {
            ByteUnit::KB | ByteUnit::MB | ByteUnit::GB => true,
            ByteUnit::B | ByteUnit::KiB | ByteUnit::MiB | ByteUnit::GiB => false,
        }
    }

    pub fn is_binary_unit(&self) -> bool {
        !self.is_decimal_unit()
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
            "kb" | "kB" | "KB" => Ok(ByteUnit::KB),
            "kib" | "KiB" => Ok(ByteUnit::KiB),
            "mb" | "mB" | "MB" => Ok(ByteUnit::MB),
            "mib" | "MiB" => Ok(ByteUnit::MiB),
            "gb" | "gB" | "GB" => Ok(ByteUnit::GB),
            "gib" | "GiB" => Ok(ByteUnit::GiB),
            _ => Err(Error::new(clap::error::ErrorKind::ValueValidation)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
pub enum UnicodeEncoding {
    Utf8,
    Utf16,
    Utf32,
}

impl FromStr for UnicodeEncoding {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "utf_8" | "utf-8" | "utf8" | "8" => Ok(UnicodeEncoding::Utf8),
            "utf_16" | "utf-16" | "utf16" | "16" => Ok(UnicodeEncoding::Utf16),
            "utf_32" | "utf-32" | "utf32" | "32" => Ok(UnicodeEncoding::Utf32),
            _ => Err(Error::new(clap::error::ErrorKind::ValueValidation)),
        }
    }
}

impl Display for UnicodeEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnicodeEncoding::Utf8 => write!(f, "UTF-8"),
            UnicodeEncoding::Utf16 => write!(f, "UTF-16"),
            UnicodeEncoding::Utf32 => write!(f, "UTF-32"),
        }
    }
}
