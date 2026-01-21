use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RdsGroup {
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub d: u16,
}

pub struct RdsGroupIterator<R: BufRead> {
    lines: R,
}

impl<R: BufRead> RdsGroupIterator<R> {
    pub fn new(reader: R) -> Self {
        Self { lines: reader }
    }
}

impl<R: BufRead> Iterator for RdsGroupIterator<R> {
    type Item = io::Result<RdsGroup>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut line_buf = String::new();
            match self.lines.read_line(&mut line_buf) {
                Ok(0) => return None, // EOF
                Ok(_) => {
                    let line = line_buf.trim();

                    if line.is_empty()
                        || line.starts_with('%')
                        || line.starts_with('<')
                        || !line.starts_with(|c: char| c.is_ascii_hexdigit())
                    {
                        continue;
                    }

                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() < 4 {
                        eprintln!("Warning: skipping short line: {}", line);
                        continue;
                    }

                    let parse_hex = |s: &str| -> Option<u16> {
                        u16::from_str_radix(s, 16)
                            .map_err(|e| {
                                eprintln!("Hex parse error on '{}': {}", s, e);
                                e
                            })
                            .ok()
                    };

                    match (
                        parse_hex(parts[0]),
                        parse_hex(parts[1]),
                        parse_hex(parts[2]),
                        parse_hex(parts[3]),
                    ) {
                        (Some(a), Some(b), Some(c), Some(d)) => {
                            return Some(Ok(RdsGroup { a, b, c, d }));
                        }
                        _ => {
                            eprintln!("Skipping invalid hex line: {}", line);
                            continue;
                        }
                    }
                }
                Err(e) => return Some(Err(e)),
            }
        }
    }
}

pub fn read_rds_groups<P: AsRef<Path>>(
    path: P,
) -> io::Result<impl Iterator<Item = io::Result<RdsGroup>>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(RdsGroupIterator::new(reader))
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let reader: Box<dyn BufRead> = if args.len() == 1 {
        println!("Reading RDS groups from stdin...");
        Box::new(BufReader::new(io::stdin()))
    } else if args.len() == 2 {
        let path = &args[1];
        println!("Reading RDS groups from: {}", path);
        Box::new(BufReader::new(File::open(path)?))
    } else {
        eprintln!("Usage: {} [path_to_rds_file]", args[0]);
        eprintln!("(omit path to read from stdin)");
        std::process::exit(1);
    };

    for group_result in RdsGroupIterator::new(reader) {
        match group_result {
            Ok(group) => {
                println!(
                    "A:{:04X} B:{:04X} C:{:04X} D:{:04X}",
                    group.a, group.b, group.c, group.d
                );
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}
