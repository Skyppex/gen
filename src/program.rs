use std::{io::Write, num::NonZeroUsize};

use crate::args::{ByteSize, Command, FloatRange, GenArgs, IntRange, UnicodeEncoding, UuidVersion};
use rand::Rng;
use random_string::{charsets, generate, generate_rng};
use uuid::Uuid;

pub fn run<T: Write>(args: GenArgs, writer: T) {
    match args.commands {
        Command::Int { range } => generate_int(range, writer),
        Command::Float { range } => generate_float(range, writer),
        Command::Uuid { version } => generate_uuid(version, writer),
        Command::Url {
            length,
            path,
            query,
        } => generate_url(length, path, query, writer),
        Command::Ascii {
            charset,
            printable_only,
            exclude,
            exclude_codes,
            size,
        } => generate_ascii(
            charset,
            printable_only,
            exclude,
            exclude_codes,
            size,
            writer,
        ),
        Command::Unicode {
            encoding,
            charset,
            exclude,
            size,
        } => generate_unicode(encoding, charset, exclude, size, writer),
    }
}

fn generate_int<T: Write>(range: Option<IntRange>, mut writer: T) {
    let mut rng = rand::thread_rng();
    let min = range.clone().map_or(0, |r| r.min);
    let max = range.map_or(100, |r| r.max);
    write!(writer, "{}", rng.gen_range(min..=max)).expect("Failed to write to output");
    writer.flush().unwrap();
}

fn generate_float<T: Write>(range: Option<FloatRange>, mut writer: T) {
    let mut rng = rand::thread_rng();
    let min = range.clone().map_or(0.0, |r| r.min);
    let max = range.map_or(1.0, |r| r.max);
    write!(writer, "{}", rng.gen_range(min..=max)).expect("Failed to write to output");
    writer.flush().unwrap();
}

fn generate_uuid<T: Write>(version: Option<UuidVersion>, mut writer: T) {
    write!(
        writer,
        "{}",
        match version {
            Some(UuidVersion::Empty) => Uuid::nil().to_string(),
            Some(UuidVersion::Max) => Uuid::max().to_string(),
            Some(UuidVersion::V4) | None => Uuid::new_v4().to_string(),
        }
    )
    .expect("Failed to write to output");

    writer.flush().unwrap();
}

fn generate_url<T: Write>(
    length: Option<usize>,
    path: Option<Option<u8>>,
    query: bool,
    mut writer: T,
) {
    let protocol = "https".to_owned();
    let domain = gen_str(length) + "." + &gen_str(Some(3));
    let paths = if let Some(p) = path {
        let mut paths = Vec::new();

        for _ in 0..(p.unwrap_or(1)) {
            paths.push(gen_str(length));
        }

        Some(paths)
    } else {
        None
    };

    match (paths, query) {
        (None, false) => write!(writer, "{}://{}", protocol, domain),
        (Some(p), false) => write!(writer, "{}://{}/{}", protocol, domain, p.join("/")),
        (None, true) => write!(writer, "{}://{}/?{}", protocol, domain, gen_str(length)),
        (Some(p), true) => {
            write!(
                writer,
                "{}://{}/{}?{}",
                protocol,
                domain,
                p.join("/"),
                gen_str(length)
            )
        }
    }
    .expect("Failed to write to output");

    writer.flush().unwrap();
}

fn generate_ascii<T: Write>(
    charset: Option<String>,
    printable_only: bool,
    exclude: Option<String>,
    exclude_codes: Option<Vec<u8>>,
    size: ByteSize,
    mut writer: T,
) {
    let num_threads = num_cpus::get();
    let total_size = size.to_bytes();
    let chunk_size = total_size / num_threads;

    let ascii_chars = if printable_only {
        (32..127).filter_map(std::char::from_u32).collect()
    } else {
        (0..128).filter_map(std::char::from_u32).collect()
    };

    let mut chars = charset
        .unwrap_or(ascii_chars)
        .chars()
        .collect::<Vec<char>>();

    match (exclude, exclude_codes) {
        (Some(e), None) => chars.retain(|c| !e.chars().collect::<Vec<char>>().contains(c)),
        (None, Some(e)) => chars.retain(|c| {
            !e.iter()
                .map(|c| *c as char)
                .collect::<Vec<char>>()
                .contains(c)
        }),
        (Some(e1), Some(e2)) => chars.retain(|c| {
            !e1.chars().collect::<Vec<char>>().contains(c)
                && !e2
                    .iter()
                    .map(|c| *c as char)
                    .collect::<Vec<char>>()
                    .contains(c)
        }),
        (None, None) => {}
    }

    if chars.is_empty() {
        panic!("Charset cannot be empty");
    }

    let mut rng = rand::thread_rng();

    (0..total_size).for_each(|_| {
        let idx = rng.gen_range(0..chars.len());
        write!(writer, "{}", chars[idx]).expect("Failed to write to output");
    });

    writer.flush().unwrap();
}

fn generate_unicode<T: Write>(
    encoding: UnicodeEncoding,
    charset: Option<String>,
    exclude: Option<String>,
    size: ByteSize,
    mut writer: T,
) {
    let unicode_chars: Vec<char> = (0..=0x10FFFF).filter_map(std::char::from_u32).collect();

    let mut chars = charset
        .unwrap_or_else(|| unicode_chars.iter().collect())
        .chars()
        .collect::<Vec<char>>();

    if let Some(e) = exclude {
        chars.retain(|c| !e.chars().collect::<Vec<char>>().contains(c));
    }

    if chars.is_empty() {
        panic!("Charset cannot be empty");
    }

    let mut rng = rand::thread_rng();

    let target_bytes = size.to_bytes();
    let mut current_bytes = 0;

    while current_bytes < target_bytes {
        let idx = rng.gen_range(0..chars.len());
        let ch = chars[idx];

        let ch_bytes = match encoding {
            UnicodeEncoding::Utf8 => ch.len_utf8(),
            UnicodeEncoding::Utf16 => ch.len_utf16() * 2, // Each UTF-16 unit is 2 bytes
            UnicodeEncoding::Utf32 => 4,                  // Each UTF-32 character is 4 bytes
        };

        if current_bytes + ch_bytes <= target_bytes {
            write!(writer, "{}", ch).expect("Failed to write to output");
            current_bytes += ch_bytes;
        }
    }

    writer.flush().unwrap();
}

fn gen_str(length: Option<usize>) -> String {
    match length {
        Some(l) => generate(l, charsets::ALPHA_LOWER),
        None => generate_rng(5..15, charsets::ALPHA_LOWER),
    }
}

#[cfg(test)]
mod tests {}
