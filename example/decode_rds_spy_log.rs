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
    for group_result in RdsGroupIterator::new(reader) {
        match group_result {
            Ok(group) => {
                println!("{:?}", group);
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
                    println!("Processing file: {}", path.display());
                    let file = match File::open(path) {
                        Ok(f) => f,
                        Err(e) => {
                            eprintln!("Failed to open {}: {}", path.display(), e);
                            continue;
                        }
                    };
                    if let Err(e) = process_reader(BufReader::new(file)) {
                        eprintln!("Error processing {}: {}", path.display(), e);
                    }
                }
            }
        }
    }

    Ok(())
}
