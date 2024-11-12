use std::{
    io::Write,
    num::NonZeroUsize,
    sync::{Arc, Mutex},
    thread,
};

use crate::args::{ByteSize, Command, FloatRange, GenArgs, IntRange, UnicodeEncoding, UuidVersion};
use rand::Rng;
use random_string::{charsets, generate, generate_rng};
use uuid::Uuid;

pub fn run<T: Write + Send + 'static>(args: GenArgs, writer: Arc<Mutex<T>>) {
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
            size,
            charset,
            printable_only,
            exclude,
            exclude_codes,
            threads,
        } => generate_ascii(
            size,
            charset,
            printable_only,
            exclude,
            exclude_codes,
            threads,
            writer,
        ),
        Command::Unicode {
            size,
            encoding,
            charset,
            exclude,
            threads,
        } => generate_unicode(size, encoding, charset, exclude, threads, writer),
    }
}

fn generate_int<T: Write>(range: Option<IntRange>, writer: Arc<Mutex<T>>) {
    let mut writer = writer.lock().unwrap();
    let mut rng = rand::thread_rng();
    let min = range.clone().map_or(0, |r| r.min);
    let max = range.map_or(100, |r| r.max);
    write!(writer, "{}", rng.gen_range(min..=max)).expect("Failed to write to output");
    writer.flush().unwrap();
}

fn generate_float<T: Write>(range: Option<FloatRange>, writer: Arc<Mutex<T>>) {
    let mut writer = writer.lock().unwrap();
    let mut rng = rand::thread_rng();
    let min = range.clone().map_or(0.0, |r| r.min);
    let max = range.map_or(1.0, |r| r.max);
    write!(writer, "{}", rng.gen_range(min..=max)).expect("Failed to write to output");
    writer.flush().unwrap();
}

fn generate_uuid<T: Write>(version: Option<UuidVersion>, writer: Arc<Mutex<T>>) {
    let mut writer = writer.lock().unwrap();
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
    writer: Arc<Mutex<T>>,
) {
    let mut writer = writer.lock().unwrap();
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

fn generate_ascii<T: Write + Send + 'static>(
    size: ByteSize,
    charset: Option<String>,
    printable_only: bool,
    exclude: Option<String>,
    exclude_codes: Option<Vec<u8>>,
    threads: Option<NonZeroUsize>,
    writer: Arc<Mutex<T>>,
) {
    let ascii_chars: Vec<char> = if printable_only {
        (32..127).filter_map(std::char::from_u32).collect()
    } else {
        (0..128).filter_map(std::char::from_u32).collect()
    };

    let mut chars: Vec<char> = charset
        .unwrap_or_else(|| ascii_chars.iter().collect())
        .chars()
        .collect();

    if let Some(exclude) = exclude {
        chars.retain(|c| !exclude.contains(*c));
    }

    if let Some(exclude_codes) = exclude_codes {
        chars.retain(|c| !exclude_codes.contains(&(*c as u8)));
    }

    if chars.is_empty() {
        panic!("Charset cannot be empty");
    }

    let num_threads = threads.map(|t| t.get()).unwrap_or_else(|| num_cpus::get());
    let total_size = size.to_bytes();
    let chunk_size = total_size / num_threads;
    let remainder = total_size % num_threads;

    let chars_len = chars.len();
    let chars = Arc::new(chars);

    let mut handles = vec![];

    for i in 0..num_threads {
        let writer = Arc::clone(&writer);
        let chars = Arc::clone(&chars);

        let chunk_size = if i == num_threads - 1 {
            chunk_size + remainder
        } else {
            chunk_size
        };

        let handle = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let mut buffer = Vec::with_capacity(chunk_size);

            for _ in 0..chunk_size {
                let char_index = rng.gen_range(0..chars_len);
                buffer.push(chars[char_index] as u8);
            }

            let mut writer = writer.lock().expect("Failed to lock writer");
            writer
                .write_all(&buffer)
                .expect("Failed to write to buffer");
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

fn generate_unicode<T: Write + Send + 'static>(
    size: ByteSize,
    encoding: UnicodeEncoding,
    charset: Option<String>,
    exclude: Option<String>,
    threads: Option<NonZeroUsize>,
    writer: Arc<Mutex<T>>,
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

    let num_threads = threads.map(|t| t.get()).unwrap_or_else(|| num_cpus::get());
    let total_size = size.to_bytes();
    let chunk_size = total_size / num_threads;
    let remainder = total_size % num_threads;

    let chars_len = chars.len();
    let chars = Arc::new(chars);
    let encoding = Arc::new(encoding);

    let mut handles = vec![];

    for i in 0..num_threads {
        let writer = Arc::clone(&writer);
        let chars = Arc::clone(&chars);
        let encoding = Arc::clone(&encoding);

        let chunk_size = if i == num_threads - 1 {
            chunk_size + remainder
        } else {
            chunk_size
        };

        let handle = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let mut buffer = Vec::with_capacity(chunk_size);
            let mut current_bytes = 0;

            while current_bytes < chunk_size {
                let char_index = rng.gen_range(0..chars_len);
                let ch = chars[char_index];

                let ch_bytes = match *encoding.clone() {
                    UnicodeEncoding::Utf8 => ch.len_utf8(),
                    UnicodeEncoding::Utf16 => ch.len_utf16() * 2, // Each UTF-16 unit is 2 bytes
                    UnicodeEncoding::Utf32 => 4, // Each UTF-32 character is 4 bytes
                };

                if current_bytes + ch_bytes <= chunk_size {
                    buffer.push(ch as u8);
                    current_bytes += ch_bytes;
                }
            }

            let mut writer = writer.lock().expect("Failed to lock writer");

            writer
                .write_all(&buffer)
                .expect("Failed to write to buffer");
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

fn gen_str(length: Option<usize>) -> String {
    match length {
        Some(l) => generate(l, charsets::ALPHA_LOWER),
        None => generate_rng(5..15, charsets::ALPHA_LOWER),
    }
}

#[cfg(test)]
mod tests {}
