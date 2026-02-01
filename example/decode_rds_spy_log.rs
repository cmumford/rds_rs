use log::{error, info};
use rds::{Decoder, Group, RdsData, RtVariant, rds_to_utf8_lossy};
use rdspy::RdsGroupIterator;

use std::{
    env,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => {
            // No argument → stdin
            println!("Reading RDS groups from stdin...");
            process_reader(BufReader::new(io::stdin()))?;
        }
        2 => {
            let path = Path::new(&args[1]);

            if path.is_dir() {
                println!("Scanning directory: {}", path.display());
                process_directory(path)?;
            } else if path.is_file() {
                println!("Reading RDS groups from file: {}", path.display());
                let file = File::open(path)?;
                process_reader(BufReader::new(file))?;
            } else {
                eprintln!("Error: '{}' is not a file or directory", path.display());
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Usage: {} [path]", args[0]);
            eprintln!("  path can be:");
            eprintln!("    - omitted          → read from stdin");
            eprintln!("    - a file           → process single .rds / .spy file");
            eprintln!("    - a directory      → recursively process all .rds and .spy files");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn process_reader<R: BufRead + 'static>(reader: R) -> io::Result<()> {
    let mut last_rt = String::new();

    let mut rds_data = RdsData::default();
    let mut decoder = Decoder::new(false);
    for group_result in RdsGroupIterator::new(reader) {
        match group_result {
            Ok(read_group) => {
                let group = Group {
                    a: read_group.a,
                    b: read_group.b,
                    c: read_group.c,
                    d: read_group.d,
                };
                decoder.decode(&group, &mut rds_data);
                if rds_data.valid.rt() {
                    let rt = match rds_data.rt.decode_rt {
                        RtVariant::A => &rds_data.rt.a,
                        RtVariant::B => &rds_data.rt.b,
                    };
                    let text = rds_to_utf8_lossy(&rt.display);
                    let trimmed = text.trim_end();
                    if last_rt != trimmed {
                        print!("RT: {:?}", trimmed);
                        last_rt = trimmed.to_string();
                        if rds_data.valid.ptyn() {
                            print!(
                                " PTYN: {:?}",
                                rds_to_utf8_lossy(&rds_data.ptyn.display).trim_end()
                            );
                        }
                        // Too verbose
                        // if rds_data.valid.pi_code() {
                        //     print!(" PI: {:?}", rds_data.program_information);
                        // }
                        if rds_data.valid.ps() {
                            print!(
                                " PS: {:?}",
                                rds_to_utf8_lossy(&rds_data.ps.display).trim_end()
                            );
                        }
                        if rds_data.valid.clock() {
                            let c = &rds_data.clock;
                            print!(
                                " CLOCK: {:04}/{:02}/{:02} {:02}:{:02}",
                                c.year(),
                                c.month(),
                                c.day(),
                                c.hour,
                                c.minute
                            );
                        }
                        if rds_data.valid.ms() {
                            print!(" MS: {:?}", rds_data.content);
                        }

                        println!("");
                    }
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    Ok(())
}

fn process_directory(dir: &Path) -> io::Result<()> {
    for entry in walkdir::WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "rds" || ext == "spy" {
                    info!("Processing file: {}", path.display());
                    let file = match File::open(path) {
                        Ok(f) => f,
                        Err(e) => {
                            eprintln!("Failed to open {}: {}", path.display(), e);
                            continue;
                        }
                    };
                    if let Err(e) = process_reader(BufReader::new(file)) {
                        error!("Error processing {}: {}", path.display(), e);
                    }
                }
            }
        }
    }

    Ok(())
}
