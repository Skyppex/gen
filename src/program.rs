#![allow(clippy::too_many_arguments)]

use std::{
    io::{Result, Write},
    num::NonZeroUsize,
    sync::{Arc, Mutex},
    thread,
};

use crate::args::{
    ByteSize, ByteUnit, Command, FloatRange, GenArgs, IntRange, Size, UnicodeEncoding, UuidVersion,
};
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use random_string::{charsets, generate, generate_rng};
use uuid::Uuid;

pub fn run<T: Write + Send + 'static>(args: GenArgs, writer: Arc<Mutex<T>>) {
    match args.commands {
        Command::Int {
            range,
            amount,
            threads,
            buf_size,
            progress,
        } => generate_int(
            range,
            amount,
            threads,
            buf_size,
            progress,
            writer,
            args.daemon,
        ),
        Command::Float {
            range,
            amount,
            threads,
            buf_size,
            progress,
        } => generate_float(
            range,
            amount,
            threads,
            buf_size,
            progress,
            writer,
            args.daemon,
        ),
        Command::Uuid {
            version,
            amount,
            threads,
            buf_size,
            progress,
        } => generate_uuid(
            version,
            amount,
            threads,
            buf_size,
            progress,
            writer,
            args.daemon,
        ),
        Command::Url {
            length,
            resource: path,
            query,
            amount,
            threads,
            buf_size,
            progress,
        } => generate_url(
            length,
            path,
            query,
            amount,
            threads,
            buf_size,
            progress,
            writer,
            args.daemon,
        ),
        Command::Ascii {
            size,
            charset,
            printable_only,
            exclude,
            exclude_codes,
            threads,
            buf_size,
            progress,
        } => generate_ascii(
            size,
            charset,
            printable_only,
            exclude,
            exclude_codes,
            threads,
            buf_size,
            progress,
            writer,
            args.daemon,
        ),
        Command::Unicode {
            size,
            encoding,
            charset,
            exclude,
            threads,
            buf_size,
            progress,
        } => generate_unicode(
            size,
            encoding,
            charset,
            exclude,
            threads,
            buf_size,
            progress,
            writer,
            args.daemon,
        ),
    }
}

const SIMUL_BYTES: usize = 8;

fn generate_int<T: Write + Send + 'static>(
    range: Option<IntRange>,
    amount: Option<Size>,
    threads: Option<NonZeroUsize>,
    buf_size: Option<Size>,
    progress: bool,
    writer: Arc<Mutex<T>>,
    daemon: bool,
) {
    let min = range.clone().map_or(0, |r| r.min);
    let max = range.map_or(99, |r| r.max);

    let infinite = daemon && amount.is_none();
    let amount = amount.map_or(1, |a| a.get());
    let num_threads = threads.map(|t| t.get()).unwrap_or_else(num_cpus::get);
    let total_size = amount;
    let full_chunks = total_size / num_threads;
    let remaining_bytes = total_size % num_threads;

    let mut chunks = vec![full_chunks; num_threads];

    if remaining_bytes > 0 {
        let additional_chunks = remaining_bytes;

        for i in 0..additional_chunks {
            chunks[i % num_threads] += 1;
        }
    }

    // Remove one from the first chunk to account
    // for the last write without a newline
    chunks[0] -= 1;

    let progress_bar = Arc::new(if progress && !daemon {
        Some(create_progress_bar_amount(amount as u64))
    } else {
        None
    });

    let buf_size = Arc::new(buf_size);

    let mut handles = vec![];

    for chunk_size in chunks {
        if chunk_size == 0 && !infinite {
            continue;
        }

        let writer = Arc::clone(&writer);
        let progress_bar = Arc::clone(&progress_bar);

        let buf_size = buf_size.map(|b| b.get()).unwrap_or(if chunk_size > 1000 {
            chunk_size / num_threads
        } else {
            chunk_size
        });

        let handle = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let mut gen_func = || rng.gen_range(min..=max);

            if infinite {
                let buf_size = 1024;
                let mut buffer = Vec::with_capacity(buf_size);

                loop {
                    generate_random_value(buf_size, &mut gen_func, &mut buffer, &progress_bar);

                    if writeln(&writer, &mut buffer).is_err() {
                        break;
                    }
                }

                return;
            }

            let mut buffer = Vec::with_capacity(buf_size);

            let rounds = chunk_size / buf_size;
            let remainder = chunk_size % buf_size;

            for _ in 0..rounds {
                generate_random_value(buf_size, &mut gen_func, &mut buffer, &progress_bar);
                writeln(&writer, &mut buffer).expect("Failed to write to buffer");
            }

            if remainder == 0 {
                return;
            }

            generate_random_value(remainder, &mut gen_func, &mut buffer, &progress_bar);
            writeln(&writer, &mut buffer).expect("Failed to write to buffer");
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    if infinite {
        return;
    }

    let mut rng = rand::thread_rng();

    write(&writer, rng.gen_range(min..=max).to_string().as_bytes());
}

fn generate_float<T: Write + Send + 'static>(
    range: Option<FloatRange>,
    amount: Option<Size>,
    threads: Option<NonZeroUsize>,
    buf_size: Option<Size>,
    progress: bool,
    writer: Arc<Mutex<T>>,
    daemon: bool,
) {
    let min = range.clone().map_or(0.0, |r| r.min);
    let max = range.map_or(1.0, |r| r.max);

    let infinite = daemon && amount.is_none();
    let amount = amount.map_or(1, |a| a.get());
    let num_threads = threads.map(|t| t.get()).unwrap_or_else(num_cpus::get);
    let total_size = amount;
    let full_chunks = total_size / num_threads;
    let remaining_bytes = total_size % num_threads;

    let mut chunks = vec![full_chunks; num_threads];

    if remaining_bytes > 0 {
        let additional_chunks = remaining_bytes;

        for i in 0..additional_chunks {
            chunks[i % num_threads] += 1;
        }
    }

    // Remove one from the first chunk to account
    // for the last write without a newline
    chunks[0] -= 1;

    let progress_bar = Arc::new(if progress && !daemon {
        Some(create_progress_bar_amount(amount as u64))
    } else {
        None
    });

    let buf_size = Arc::new(buf_size);

    let mut handles = vec![];

    for chunk_size in chunks {
        if chunk_size == 0 && !infinite {
            continue;
        }

        let writer = Arc::clone(&writer);
        let progress_bar = Arc::clone(&progress_bar);

        let buf_size = buf_size.map(|b| b.get()).unwrap_or(if chunk_size > 1000 {
            chunk_size / num_threads
        } else {
            chunk_size
        });

        let handle = thread::spawn(move || {
            let mut rng = rand::thread_rng();

            let mut gen_func = || rng.gen_range(min..=max);

            if infinite {
                let buf_size = 1024;
                let mut buffer = Vec::with_capacity(buf_size);

                loop {
                    generate_random_value(buf_size, &mut gen_func, &mut buffer, &progress_bar);

                    if writeln(&writer, &mut buffer).is_err() {
                        break;
                    }
                }

                return;
            }

            let mut buffer = Vec::with_capacity(buf_size);

            let rounds = chunk_size / buf_size;
            let remainder = chunk_size % buf_size;

            for _ in 0..rounds {
                generate_random_value(buf_size, &mut gen_func, &mut buffer, &progress_bar);
                writeln(&writer, &mut buffer).expect("Failed to write to buffer");
            }

            if remainder == 0 {
                return;
            }

            generate_random_value(remainder, &mut gen_func, &mut buffer, &progress_bar);
            writeln(&writer, &mut buffer).expect("Failed to write to buffer");
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    if infinite {
        return;
    }

    let mut rng = rand::thread_rng();

    write(&writer, rng.gen_range(min..=max).to_string().as_bytes());
}

fn generate_uuid<T: Write + Send + 'static>(
    version: Option<UuidVersion>,
    amount: Option<Size>,
    threads: Option<NonZeroUsize>,
    buf_size: Option<Size>,
    progress: bool,
    writer: Arc<Mutex<T>>,
    daemon: bool,
) {
    let infinite = daemon && amount.is_none();
    let amount = amount.map_or(1, |a| a.get());
    let num_threads = threads.map(|t| t.get()).unwrap_or_else(num_cpus::get);
    let total_size = amount;
    let full_chunks = total_size / num_threads;
    let remaining_bytes = total_size % num_threads;

    let mut chunks = vec![full_chunks; num_threads];

    if remaining_bytes > 0 {
        let additional_chunks = remaining_bytes;

        for i in 0..additional_chunks {
            chunks[i % num_threads] += 1;
        }
    }

    // Remove one from the first chunk to account
    // for the last write without a newline
    chunks[0] -= 1;

    let progress_bar = Arc::new(if progress && !daemon {
        Some(create_progress_bar_amount(amount as u64))
    } else {
        None
    });

    let buf_size = Arc::new(buf_size);

    let mut gen_func = match version {
        Some(UuidVersion::Empty) => || Uuid::nil().to_string(),
        Some(UuidVersion::Max) => || Uuid::max().to_string(),
        Some(UuidVersion::V4) | None => || Uuid::new_v4().to_string(),
    };

    let mut handles = vec![];

    for chunk_size in chunks {
        if chunk_size == 0 && !infinite {
            continue;
        }

        let writer = Arc::clone(&writer);
        let progress_bar = Arc::clone(&progress_bar);

        let buf_size = buf_size.map(|b| b.get()).unwrap_or(if chunk_size > 1000 {
            chunk_size / num_threads
        } else {
            chunk_size
        });

        let handle = thread::spawn(move || {
            if infinite {
                let buf_size = 1024;
                let mut buffer = Vec::with_capacity(buf_size);

                loop {
                    generate_random_value(buf_size, &mut gen_func, &mut buffer, &progress_bar);

                    if writeln(&writer, &mut buffer).is_err() {
                        break;
                    }
                }

                return;
            }

            let mut buffer = Vec::with_capacity(buf_size);
            let rounds = chunk_size / buf_size;
            let remainder = chunk_size % buf_size;

            for _ in 0..rounds {
                generate_random_value(buf_size, &mut gen_func, &mut buffer, &progress_bar);
                writeln(&writer, &mut buffer).expect("Failed to write to buffer");
            }

            if remainder == 0 {
                return;
            }

            generate_random_value(remainder, &mut gen_func, &mut buffer, &progress_bar);
            writeln(&writer, &mut buffer).expect("Failed to write to buffer");
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    if infinite {
        return;
    }

    write(&writer, gen_func().as_bytes());
}

fn generate_url<T: Write + Send + 'static>(
    length: Option<usize>,
    path: Option<Option<u8>>,
    query: bool,
    amount: Option<Size>,
    threads: Option<NonZeroUsize>,
    buf_size: Option<Size>,
    progress: bool,
    writer: Arc<Mutex<T>>,
    daemon: bool,
) {
    let infinite = daemon && amount.is_none();
    let amount = amount.map_or(1, |a| a.get());
    let num_threads = threads.map(|t| t.get()).unwrap_or_else(num_cpus::get);
    let total_size = amount;
    let full_chunks = total_size / num_threads;
    let remaining_bytes = total_size % num_threads;

    let mut chunks = vec![full_chunks; num_threads];

    if remaining_bytes > 0 {
        let additional_chunks = remaining_bytes;

        for i in 0..additional_chunks {
            chunks[i % num_threads] += 1;
        }
    }

    // Remove one from the first chunk to account
    // for the last write without a newline
    chunks[0] -= 1;

    let progress_bar = Arc::new(if progress && !daemon {
        Some(create_progress_bar_amount(amount as u64))
    } else {
        None
    });

    let buf_size = Arc::new(buf_size);

    let mut gen_func = move || {
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
            (None, false) => {
                format!("{protocol}://{domain}")
            }
            (Some(p), false) => {
                format!("{}://{}/{}", protocol, domain, p.join("/"))
            }
            (None, true) => {
                format!("{}://{}/?{}", protocol, domain, gen_str(length))
            }
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
    };

    let mut handles = vec![];

    for chunk_size in chunks {
        if chunk_size == 0 && !infinite {
            continue;
        }

        let writer = Arc::clone(&writer);
        let progress_bar = Arc::clone(&progress_bar);

        let buf_size = buf_size.map(|b| b.get()).unwrap_or(if chunk_size > 1000 {
            chunk_size / num_threads
        } else {
            chunk_size
        });

        let handle = thread::spawn(move || {
            if infinite {
                let buf_size = 1024;
                let mut buffer = Vec::with_capacity(buf_size);

                loop {
                    generate_random_value(buf_size, &mut gen_func, &mut buffer, &progress_bar);

                    if writeln(&writer, &mut buffer).is_err() {
                        break;
                    }
                }

                return;
            }

            let mut buffer = Vec::with_capacity(buf_size);
            let rounds = chunk_size / buf_size;
            let remainder = chunk_size % buf_size;

            for _ in 0..rounds {
                generate_random_value(buf_size, &mut gen_func, &mut buffer, &progress_bar);
                writeln(&writer, &mut buffer).expect("Failed to write to buffer");
            }

            if remainder == 0 {
                return;
            }

            generate_random_value(remainder, &mut gen_func, &mut buffer, &progress_bar);
            writeln(&writer, &mut buffer).expect("Failed to write to buffer");
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    if infinite {
        return;
    }

    write(&writer, gen_func().as_bytes());
}

fn generate_ascii<T: Write + Send + 'static>(
    size: Option<ByteSize>,
    charset: Option<String>,
    printable_only: bool,
    exclude: Option<String>,
    exclude_codes: Option<Vec<u8>>,
    threads: Option<NonZeroUsize>,
    buf_size: Option<ByteSize>,
    progress: bool,
    writer: Arc<Mutex<T>>,
    daemon: bool,
) {
    let infinite = daemon && size.is_none();

    let size = size.unwrap_or(ByteSize {
        value: 1,
        unit: ByteUnit::B,
    });

    let num_threads = threads.map(|t| t.get()).unwrap_or_else(num_cpus::get);
    let buf_size = buf_size.map(|b| b.to_bytes() / num_threads);

    if let Some(ref buf_size) = buf_size {
        if buf_size < &SIMUL_BYTES {
            panic!(
                "Buffer size after being divided by the number of threads ({num_threads}) must be {SIMUL_BYTES} or greater and divisible by {SIMUL_BYTES}"
            );
        }
    }

    let buf_size = buf_size.map(|b| b / SIMUL_BYTES * SIMUL_BYTES);

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

    let total_size = size.to_bytes();

    let full_chunks = total_size / (num_threads * SIMUL_BYTES);

    let remaining_bytes = total_size % (num_threads * SIMUL_BYTES);

    let mut chunks = vec![full_chunks; num_threads];

    if remaining_bytes >= SIMUL_BYTES {
        let additional_chunks = remaining_bytes / SIMUL_BYTES;

        for i in 0..additional_chunks {
            chunks[i % num_threads] += 1;
        }
    }

    let progress_bar = Arc::new(if progress {
        Some(create_progress_bar(
            total_size as u64,
            size.is_binary_unit(),
        ))
    } else {
        None
    });

    let chars_len = chars.len();
    let chars = Arc::new(chars);
    let buf_size = Arc::new(buf_size);

    let mut handles = vec![];

    for chunk_size in chunks {
        if chunk_size == 0 && !infinite {
            continue;
        }

        let writer = Arc::clone(&writer);
        let progress_bar = Arc::clone(&progress_bar);

        let chars = Arc::clone(&chars);
        let byte_count = chunk_size * SIMUL_BYTES;
        let buf_size = buf_size.unwrap_or(byte_count);

        let handle = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            if infinite {
                let buf_size = SIMUL_BYTES * 128;
                let mut buffer = Vec::with_capacity(buf_size);

                loop {
                    generate_random_ascii_8(
                        buf_size / SIMUL_BYTES,
                        &mut rng,
                        chars_len,
                        &mut buffer,
                        &chars,
                        &progress_bar,
                    );

                    if write_from_buffer(&writer, &mut buffer).is_err() {
                        break;
                    }
                }

                return;
            }

            let mut buffer = Vec::with_capacity(buf_size);
            let rounds = byte_count / buf_size;
            let remainder = byte_count % buf_size;

            for _ in 0..rounds {
                generate_random_ascii_8(
                    buf_size / SIMUL_BYTES,
                    &mut rng,
                    chars_len,
                    &mut buffer,
                    &chars,
                    &progress_bar,
                );

                write_from_buffer(&writer, &mut buffer).expect("Failed to write to buffer");
            }

            if remainder < SIMUL_BYTES {
                return;
            }

            let remainder_simul = remainder / SIMUL_BYTES * SIMUL_BYTES;

            generate_random_ascii_8(
                remainder_simul / SIMUL_BYTES,
                &mut rng,
                chars_len,
                &mut buffer,
                &chars,
                &progress_bar,
            );

            generate_random_ascii(
                remainder - remainder_simul,
                rng,
                chars_len,
                &mut buffer,
                chars,
                &progress_bar,
            );

            write_from_buffer(&writer, &mut buffer).expect("Failed to write to buffer");
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    if infinite {
        return;
    }

    let leftover_bytes = remaining_bytes % SIMUL_BYTES;

    if leftover_bytes > 0 {
        let rng = rand::thread_rng();
        let mut buffer = Vec::with_capacity(leftover_bytes);

        generate_random_ascii(
            leftover_bytes,
            rng,
            chars_len,
            &mut buffer,
            chars,
            &progress_bar,
        );

        write_from_buffer(&writer, &mut buffer).expect("Failed to write to buffer");
    }
}

fn generate_unicode<T: Write + Send + 'static>(
    size: Option<ByteSize>,
    encoding: UnicodeEncoding,
    charset: Option<String>,
    exclude: Option<String>,
    threads: Option<NonZeroUsize>,
    buf_size: Option<ByteSize>,
    progress: bool,
    writer: Arc<Mutex<T>>,
    daemon: bool,
) {
    if daemon {
        panic!("--daemon option is not supported for unicode generation yet");
    }

    let min_byte_size = match encoding {
        UnicodeEncoding::Utf8 => 1,
        UnicodeEncoding::Utf16 => 2,
        UnicodeEncoding::Utf32 => 4,
    };

    let size = size.unwrap_or(ByteSize {
        value: min_byte_size,
        unit: ByteUnit::B,
    });

    let num_threads = threads.map(|t| t.get()).unwrap_or_else(num_cpus::get);
    let buf_size = buf_size.map(|b| b.to_bytes() / num_threads);

    if let Some(ref buf_size) = buf_size {
        if buf_size < &min_byte_size {
            panic!(
                "Buffer size after being divided by the number of threads ({num_threads}) must be {min_byte_size} or greater and divisible by {min_byte_size}"
            );
        }
    }

    let buf_size = buf_size.map(|b| b / min_byte_size * min_byte_size);

    let total_size = size.to_bytes();

    if total_size < min_byte_size {
        panic!(
            "Size too small for encoding.\nMinimum size for {encoding} encoding is {min_byte_size} bytes"
        );
    }

    if total_size % min_byte_size != 0 {
        panic!(
            "Size must be divisible by {min_byte_size} for {encoding} encoding"
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

    let base_size = total_size / (num_threads * min_byte_size);
    let remainder = (total_size / min_byte_size) % num_threads;

    let mut chunks = vec![base_size * min_byte_size; num_threads];

    for chunk in chunks.iter_mut().take(remainder) {
        *chunk += min_byte_size;
    }

    let progress_bar = Arc::new(if progress {
        Some(create_progress_bar(
            total_size as u64,
            size.is_binary_unit(),
        ))
    } else {
        None
    });

    let chars_len = chars.len();
    let chars = Arc::new(chars);
    let encoding = Arc::new(encoding);
    let buf_size = Arc::new(buf_size);

    let mut handles = vec![];

    for chunk_size in chunks {
        if chunk_size == 0 {
            continue;
        }

        let writer = Arc::clone(&writer);
        let progress_bar = Arc::clone(&progress_bar);

        let chars = Arc::clone(&chars);
        let encoding = Arc::clone(&encoding);
        let buf_size = buf_size.unwrap_or(chunk_size);

        let handle = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let mut buffer = Vec::with_capacity(buf_size);
            let rounds = chunk_size / buf_size;
            let remainder = chunk_size % buf_size;
            let mut current_bytes = 0;

            for _ in 0..rounds {
                generate_random_unicode(
                    &encoding,
                    &mut current_bytes,
                    buf_size,
                    &mut rng,
                    chars_len,
                    &chars,
                    &mut buffer,
                    &progress_bar,
                );

                write_from_buffer(&writer, &mut buffer).expect("Failed to write to buffer");
                current_bytes = 0;
            }

            if remainder < min_byte_size {
                return;
            }

            generate_random_unicode(
                &encoding,
                &mut current_bytes,
                remainder,
                &mut rng,
                chars_len,
                &chars,
                &mut buffer,
                &progress_bar,
            );

            write_from_buffer(&writer, &mut buffer).expect("Failed to write to buffer");
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

#[inline(always)]
fn generate_random_value<T, F: FnMut() -> T>(
    amount: usize,
    gen_func: &mut F,
    buffer: &mut Vec<T>,
    progress_bar: &Arc<Option<ProgressBar>>,
) {
    for _ in 0..amount {
        let value = gen_func();
        buffer.push(value);

        if let Some(progress_bar) = progress_bar.as_ref().clone() {
            progress_bar.inc(1);
        }
    }
}

#[inline(always)]
fn generate_random_ascii_8(
    longs: usize,
    rng: &mut rand::prelude::ThreadRng,
    chars_len: usize,
    buffer: &mut Vec<u8>,
    chars: &Arc<Vec<char>>,
    progress_bar: &Arc<Option<ProgressBar>>,
) {
    for _ in 0..longs {
        let num = rng.gen::<u64>();
        let char_indices = num.to_ne_bytes().map(|b| (b as usize) % chars_len);

        for char_index in char_indices {
            buffer.push(chars[char_index] as u8);
        }

        if let Some(progress_bar) = progress_bar.as_ref().clone() {
            progress_bar.inc(SIMUL_BYTES as u64);
        }
    }
}

#[inline(always)]
fn generate_random_ascii(
    bytes: usize,
    mut rng: rand::prelude::ThreadRng,
    chars_len: usize,
    buffer: &mut Vec<u8>,
    chars: Arc<Vec<char>>,
    progress_bar: &Arc<Option<ProgressBar>>,
) {
    for _ in 0..bytes {
        let num = rng.gen_range(0..chars_len);
        buffer.push(chars[num] as u8);

        if let Some(progress_bar) = progress_bar.as_ref().clone() {
            progress_bar.inc(SIMUL_BYTES as u64);
        }
    }
}

#[inline(always)]
fn write<T: Write>(writer: &Arc<Mutex<T>>, content: &[u8]) {
    let mut writer = writer.lock().expect("Failed to lock writer");
    writer
        .write_all(content)
        .expect("Failed to write to buffer");
}

#[inline(always)]
fn writeln<T: Write, U: ToString>(writer: &Arc<Mutex<T>>, buffer: &mut Vec<U>) -> Result<()> {
    let mut writer = writer.lock().expect("Failed to lock writer");

    writer.write_all(
        buffer
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join("\n")
            .as_bytes(),
    )?;

    writer.write_all(b"\n")?;

    buffer.clear();
    Ok(())
}

#[inline(always)]
fn write_from_buffer<T: Write>(writer: &Arc<Mutex<T>>, buffer: &mut Vec<u8>) -> Result<()> {
    let mut writer = writer.lock().expect("Failed to lock writer");
    writer.write_all(buffer)?;
    buffer.clear();
    Ok(())
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
    progress_bar: &Arc<Option<ProgressBar>>,
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

                    if let Some(progress_bar) = progress_bar.as_ref().clone() {
                        progress_bar.inc(len as u64);
                    }
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

                    if let Some(progress_bar) = progress_bar.as_ref().clone() {
                        progress_bar.inc(len as u64);
                    }
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

                if let Some(progress_bar) = progress_bar.as_ref().clone() {
                    progress_bar.inc(LEN as u64);
                }
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

fn create_progress_bar(total_size: u64, is_binary_bytes: bool) -> ProgressBar {
    let progress_bar = ProgressBar::new(total_size);

    let style = ProgressStyle::default_bar();

    let style = if is_binary_bytes {
        style
            .template(
                "{percent}% {bar:40.cyan/blue} {bytes_per_sec:.green} {bytes:.yellow}/{total_bytes:.magenta} ({eta:.cyan})",
            )
            .unwrap()
    } else {
        style
            .template(
                "{percent}% {bar:40.cyan/blue} {decimal_bytes_per_sec:.green} {decimal_bytes:.yellow}/{decimal_total_bytes:.magenta} ({eta:.cyan})"
            )
            .unwrap()
    };

    progress_bar.set_style(style);

    progress_bar
}

fn create_progress_bar_amount(total_size: u64) -> ProgressBar {
    let progress_bar = ProgressBar::new(total_size);

    let style = ProgressStyle::default_bar()
        .template(
            "{percent}% {bar:40.cyan/blue} {human_pos:.yellow}/{human_len:.magenta} ({eta:.cyan})",
        )
        .unwrap();

    progress_bar.set_style(style);

    progress_bar
}

#[cfg(test)]
mod tests {}
