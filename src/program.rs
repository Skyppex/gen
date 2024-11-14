#![allow(clippy::too_many_arguments)]

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
            buf_size,
        } => generate_ascii(
            size,
            charset,
            printable_only,
            exclude,
            exclude_codes,
            threads,
            buf_size,
            writer,
        ),
        Command::Unicode {
            size,
            encoding,
            charset,
            exclude,
            threads,
            buf_size,
        } => generate_unicode(size, encoding, charset, exclude, threads, buf_size, writer),
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
    buf_size: Option<ByteSize>,
    writer: Arc<Mutex<T>>,
) {
    if let Some(ref buf_size) = buf_size {
        if buf_size.to_bytes() % 8 != 0 {
            panic!("Buffer size must be divisible by 8");
        }
    }

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

    let num_threads = threads.map(|t| t.get()).unwrap_or_else(num_cpus::get);
    let total_size = size.to_bytes();

    const SIMUL_BYTES: usize = 8;

    let full_chunks = total_size / (num_threads * SIMUL_BYTES);

    let remaining_bytes = total_size % (num_threads * SIMUL_BYTES);

    let mut chunks = vec![full_chunks; num_threads];

    if remaining_bytes >= SIMUL_BYTES {
        let additional_chunks = remaining_bytes / SIMUL_BYTES;

        for i in 0..additional_chunks {
            chunks[i % num_threads] += 1;
        }
    }

    let chars_len = chars.len();
    let chars = Arc::new(chars);
    let buf_size = Arc::new(buf_size);

    let mut handles = vec![];

    println!("{num_threads}");

    for chunk_size in chunks {
        let writer = Arc::clone(&writer);
        let chars = Arc::clone(&chars);
        let buf_size = Arc::clone(&buf_size)
            .as_ref()
            .clone()
            .map(|b| b.to_bytes())
            .unwrap_or(chunk_size * SIMUL_BYTES);

        println!("buf: {buf_size}");
        println!("chunk: {chunk_size}");

        let handle = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let mut buffer = Vec::with_capacity(buf_size);
            let mut remaining = buf_size;

            while remaining >= SIMUL_BYTES && remaining >= buf_size {
                println!("Remaining: {remaining}");

                let rng: &mut rand::prelude::ThreadRng = &mut rng;
                let chars: &Arc<Vec<char>> = &chars;

                for _ in 0..buf_size / SIMUL_BYTES {
                    let num = rng.gen::<u64>();
                    let char_indices = num.to_ne_bytes().map(|b| (b as usize) % chars_len);

                    for char_index in char_indices {
                        buffer.push(chars[char_index] as u8);
                    }
                }

                remaining -= buf_size;

                write_from_buffer(&writer, &mut buffer);
            }

            if remaining < SIMUL_BYTES {
                return;
            }

            println!("Last remaining: {remaining}");
            {
                let rng: &mut rand::prelude::ThreadRng = &mut rng;
                let buffer: &mut Vec<u8> = &mut buffer;
                let chars: &Arc<Vec<char>> = &chars;

                for _ in 0..chunk_size {
                    let num = rng.gen::<u64>();
                    let char_indices = num.to_ne_bytes().map(|b| (b as usize) % chars_len);

                    for char_index in char_indices {
                        buffer.push(chars[char_index] as u8);
                    }
                }
            };
            write_from_buffer(&writer, &mut buffer);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    let leftover_bytes = remaining_bytes % SIMUL_BYTES;

    if leftover_bytes > 0 {
        let rng = rand::thread_rng();
        let mut buffer = Vec::with_capacity(leftover_bytes);
        generate_random_ascii(leftover_bytes, rng, chars_len, &mut buffer, chars);
        write_from_buffer(&writer, &mut buffer);
    }
}

fn generate_unicode<T: Write + Send + 'static>(
    size: ByteSize,
    encoding: UnicodeEncoding,
    charset: Option<String>,
    exclude: Option<String>,
    threads: Option<NonZeroUsize>,
    buf_size: Option<ByteSize>,
    writer: Arc<Mutex<T>>,
) {
    if let Some(ref buf_size) = buf_size {
        if buf_size.to_bytes() % 8 != 0 {
            panic!("Buffer size must be divisible by 8");
        }
    }

    let total_size = size.to_bytes();

    let min_byte_size = match encoding {
        UnicodeEncoding::Utf8 => 1,
        UnicodeEncoding::Utf16 => 2,
        UnicodeEncoding::Utf32 => 4,
    };

    if total_size < min_byte_size {
        panic!(
            "Size too small for encoding.\nMinimum size for {} encoding is {} bytes",
            encoding, min_byte_size
        );
    }

    if total_size % min_byte_size != 0 {
        panic!(
            "Size must be divisible by {} for {} encoding",
            min_byte_size, encoding
        );
    }

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

    let num_threads = threads.map(|t| t.get()).unwrap_or_else(num_cpus::get);

    let base_size = total_size / (num_threads * min_byte_size);
    let remainder = (total_size / min_byte_size) % num_threads;

    let mut chunks = vec![base_size * min_byte_size; num_threads];

    for chunk in chunks.iter_mut().take(remainder) {
        *chunk += min_byte_size;
    }

    let chars_len = chars.len();
    let chars = Arc::new(chars);
    let encoding = Arc::new(encoding);
    let buf_size = Arc::new(buf_size);

    let mut handles = vec![];

    for chunk_size in chunks {
        let writer = Arc::clone(&writer);
        let chars = Arc::clone(&chars);
        let encoding = Arc::clone(&encoding);
        let buf_size = Arc::clone(&buf_size)
            .as_ref()
            .clone()
            .map(|b| b.to_bytes())
            .unwrap_or(chunk_size * min_byte_size);

        let handle = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let mut buffer = Vec::with_capacity(buf_size);
            let mut remaining = buf_size;
            let mut current_bytes = 0;

            while remaining >= min_byte_size && remaining >= buf_size {
                generate_random_unicode(
                    &encoding,
                    &mut current_bytes,
                    chunk_size,
                    &mut rng,
                    chars_len,
                    &chars,
                    &mut buffer,
                );

                remaining -= buf_size;
                write_from_buffer(&writer, &mut buffer);
            }

            if remaining < min_byte_size {
                return;
            }

            println!("Last remaining: {remaining}");
            generate_random_unicode(
                &encoding,
                &mut current_bytes,
                chunk_size,
                &mut rng,
                chars_len,
                &chars,
                &mut buffer,
            );

            write_from_buffer(&writer, &mut buffer);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

#[inline(always)]
fn generate_random_ascii(
    leftover_bytes: usize,
    mut rng: rand::prelude::ThreadRng,
    chars_len: usize,
    buffer: &mut Vec<u8>,
    chars: Arc<Vec<char>>,
) {
    for _ in 0..leftover_bytes {
        let num = rng.gen_range(0..chars_len);
        buffer.push(chars[num] as u8);
    }
}

#[inline(always)]
fn write_from_buffer<T: Write>(writer: &Arc<Mutex<T>>, buffer: &mut Vec<u8>) {
    println!("buf len: {}", buffer.len());
    let mut writer = writer.lock().expect("Failed to lock writer");
    writer.write_all(buffer).expect("Failed to write to buffer");
    buffer.clear();
}

#[inline(always)]
fn generate_random_unicode(
    encoding: &Arc<UnicodeEncoding>,
    current_bytes: &mut usize,
    chunk_size: usize,
    rng: &mut rand::prelude::ThreadRng,
    chars_len: usize,
    chars: &Arc<Vec<char>>,
    buffer: &mut Vec<u8>,
) {
    match **encoding {
        UnicodeEncoding::Utf8 => {
            while *current_bytes < chunk_size {
                let char_index = rng.gen_range(0..chars_len);
                let ch = chars[char_index];

                let len = ch.len_utf8();

                if *current_bytes + len <= chunk_size {
                    let mut buf = [0; 4];
                    let bytes = ch.encode_utf8(&mut buf).as_bytes();
                    buffer.extend_from_slice(bytes);
                    *current_bytes += len;
                }
            }
        }
        UnicodeEncoding::Utf16 => {
            while *current_bytes < chunk_size {
                let char_index = rng.gen_range(0..chars_len);
                let ch = chars[char_index];

                // Each UTF-16 unit is 2 bytes
                let len = ch.len_utf16() * 2;

                if *current_bytes + len <= chunk_size {
                    let mut buf = [0; 2];
                    let bytes = ch.encode_utf16(&mut buf);

                    buffer.extend_from_slice(
                        &bytes
                            .iter()
                            .flat_map(|b| b.to_le_bytes())
                            .collect::<Vec<_>>(),
                    );

                    *current_bytes += len;
                }
            }
        }
        UnicodeEncoding::Utf32 => {
            while *current_bytes < chunk_size {
                let char_index = rng.gen_range(0..chars_len);
                let ch = chars[char_index];

                const LEN: usize = 4;

                buffer.extend_from_slice(&(ch as u32).to_le_bytes());
                *current_bytes += LEN;
            }
        }
    }
}

#[inline(always)]
fn gen_str(length: Option<usize>) -> String {
    match length {
        Some(l) => generate(l, charsets::ALPHA_LOWER),
        None => generate_rng(5..15, charsets::ALPHA_LOWER),
    }
}

#[cfg(test)]
mod tests {}
