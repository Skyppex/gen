use crate::args::{Command, FloatRange, GenArgs, IntRange, UuidVersion};
use rand::Rng;
use random_string::{charsets, generate, generate_rng};
use uuid::Uuid;

pub fn run(args: GenArgs) -> String {
    return match args.commands {
        Command::Int { range } => generate_int(range),
        Command::Float { range } => generate_float(range),
        Command::Uuid { version } => generate_uuid(version),
        Command::Url {
            length,
            path,
            query,
        } => generate_url(length, path, query),
    };
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
    return match version {
        Some(UuidVersion::Empty) => Uuid::nil().to_string(),
        Some(UuidVersion::Max) => Uuid::max().to_string(),
        Some(UuidVersion::V4) | None => Uuid::new_v4().to_string(),
    };
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
        (None, false) => return format!("{}://{}", protocol, domain),
        (Some(p), false) => return format!("{}://{}/{}", protocol, domain, p.join("/")),
        (None, true) => return format!("{}://{}/?{}", protocol, domain, gen_str(length)),
        (Some(p), true) => {
            return format!(
                "{}://{}/{}?{}",
                protocol,
                domain,
                p.join("/"),
                gen_str(length)
            )
        }
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
