use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders},
};
use rds::{Decoder, Group, RdsData};
use rdspy::RdsGroupIterator;
use std::{
    env,
    fs::File,
    io::{self, BufRead, BufReader, stdout},
    path::Path,
};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let rds_blocks: Vec<RdsData>;

    match args.len() {
        1 => {
            // No argument → stdin
            println!("Reading RDS groups from stdin...");
            rds_blocks = process_reader(BufReader::new(io::stdin()))?;
        }
        2 => {
            let path = Path::new(&args[1]);
            if path.is_file() {
                let file = File::open(path)?;
                rds_blocks = process_reader(BufReader::new(file))?;
            } else {
                eprintln!("Error: '{}' is not a file", path.display());
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Usage: {} [path]", args[0]);
            eprintln!("  path can be:");
            eprintln!("    - omitted          → read from stdin");
            eprintln!("    - a file           → process single .rds / .spy file");
            std::process::exit(1);
        }
    }

    if rds_blocks.is_empty() {
        println!("No RDS data can be decoded from the file.");
        return Ok(());
    }

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut block_idx: usize = 0;

    loop {
        let rds_data = rds_blocks.get(block_idx).unwrap();

        terminal.draw(|f| {
            let area = f.area();
            let status_title = format!(
                "RDS block {} of {} - Press 'q' to quit",
                block_idx + 1,
                rds_blocks.len()
            );
            let block = Block::default()
                .title(Span::styled(status_title, Style::default().fg(Color::Cyan)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .border_type(BorderType::Rounded);

            f.render_widget(block, area);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => {
                    break;
                }
                KeyCode::Right => {
                    if block_idx + 1 < rds_blocks.len() {
                        block_idx += 1;
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn process_reader<R: BufRead + 'static>(reader: R) -> io::Result<(Vec<RdsData>)> {
    let mut rds_blocks: Vec<RdsData> = Vec::new();

    let mut decoder = Decoder::new();
    for group_result in RdsGroupIterator::new(reader) {
        match group_result {
            Ok(group) => {
                let blocks = Group {
                    a: group.a,
                    b: group.b,
                    c: group.c,
                    d: group.d,
                };
                let mut rds_data = RdsData::default();
                decoder.decode(&blocks, &mut rds_data);
                rds_blocks.push(rds_data);
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    Ok(rds_blocks)
}
