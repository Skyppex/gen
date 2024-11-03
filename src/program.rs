use std::io::{self, BufWriter, Write};

use crate::args::{ByteSize, Command, FloatRange, GenArgs, IntRange, UuidVersion};
use rand::Rng;
use random_string::{charsets, generate, generate_rng};
use uuid::Uuid;

pub fn run(args: GenArgs) -> String {
    match args.commands {
        Command::Int { range } => generate_int(range),
        Command::Float { range } => generate_float(range),
        Command::Uuid { version } => generate_uuid(version),
        Command::Url {
            length,
            path,
            query,
        } => generate_url(length, path, query),
        Command::Ascii {
            charset,
            exclude,
            exclude_codes,
            size,
        } => generate_ascii(charset, exclude, exclude_codes, size),
        Command::Unicode {
            encoding,
            charset,
            exclude,
            size,
        } => generate_unicode(encoding, charset, exclude, size),
    }
}

fn generate_int(range: Option<IntRange>) -> String {
    let mut rng = rand::thread_rng();
    let min = range.clone().map_or(0, |r| r.min);
    let max = range.map_or(100, |r| r.max);
    rng.gen_range(min..=max).to_string()
}

fn generate_float(range: Option<FloatRange>) -> String {
    let mut rng = rand::thread_rng();
    let min = range.clone().map_or(0.0, |r| r.min);
    let max = range.map_or(1.0, |r| r.max);
    rng.gen_range(min..=max).to_string()
}

fn generate_uuid(version: Option<UuidVersion>) -> String {
    match version {
        Some(UuidVersion::Empty) => Uuid::nil().to_string(),
        Some(UuidVersion::Max) => Uuid::max().to_string(),
        Some(UuidVersion::V4) | None => Uuid::new_v4().to_string(),
    }
}

fn generate_url(length: Option<usize>, path: Option<Option<u8>>, query: bool) -> String {
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
        (None, false) => format!("{}://{}", protocol, domain),
        (Some(p), false) => format!("{}://{}/{}", protocol, domain, p.join("/")),
        (None, true) => format!("{}://{}/?{}", protocol, domain, gen_str(length)),
        (Some(p), true) => {
            format!(
                "{}://{}/{}?{}",
                protocol,
                domain,
                p.join("/"),
                gen_str(length)
            )
        }
    }
}

fn generate_ascii(
    charset: Option<String>,
    exclude: Option<String>,
    exclude_codes: Option<Vec<u8>>,
    size: ByteSize,
) -> String {
    // Define the ASCII character set
    let ascii_chars = (0..128).filter_map(std::char::from_u32).collect();

    // Use the provided charset or default to the full ASCII set
    let mut chars = charset
        .unwrap_or(ascii_chars)
        .chars()
        .collect::<Vec<char>>();

    // Exclude specified characters
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

    // Ensure the charset is not empty to avoid panic
    if chars.is_empty() {
        panic!("Charset cannot be empty");
    }

    let mut rng = rand::thread_rng();
    let stdout = io::stdout();
    let mut handle = BufWriter::new(stdout.lock());

    // Generate a random string of the specified size
    (0..size.to_bytes()).for_each(|_| {
        let idx = rng.gen_range(0..chars.len());
        write!(handle, "{}", chars[idx]).unwrap();
    });

    "".to_string()
}

fn generate_unicode(
    encoding: crate::args::UnicodeEncoding,
    charset: Option<String>,
    exclude: Option<String>,
    size: ByteSize,
) -> String {
    todo!()
}

fn gen_str(length: Option<usize>) -> String {
    match length {
        Some(l) => generate(l, charsets::ALPHA_LOWER),
        None => generate_rng(5..15, charsets::ALPHA_LOWER),
    }
}

#[cfg(test)]
mod tests {}
