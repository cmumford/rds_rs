use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{prelude::*, widgets::*};
use rds::{Decoder, Group, MAX_RADIOTEXT_LEN, PS_TEXT_LEN, RdsData, rds_to_utf8_lossy};
use rdspy::RdsGroupIterator;
use std::{
    env,
    fs::File,
    io::{self, BufRead, BufReader, stdout},
    path::Path,
};

// The RDS text may contain bytes that map to unicode characters to the required
// number of bytes to store the string may be greater than 8 or 64 (depending)
// on the field. Use double the length to be safe. It could still be longer though.
// This could be fixed by calling rds_to_utf8_required_bytes() if desired.
const PS_LEN: usize = 2 * PS_TEXT_LEN;
const RADIOTEXT_LEN: usize = 2 * MAX_RADIOTEXT_LEN;

// In your render/draw function:
fn draw_ui(f: &mut Frame, rds_data: &RdsData, num: usize, max: usize) {
    let area = f.area(); // or your chosen layout area

    let title = format!("  RDS Viewer. Block {} of {}  ", num, max);
    let outer_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner_area = outer_block.inner(area);
    f.render_widget(outer_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // RTA row
            Constraint::Length(1), // RTB row
            Constraint::Length(1), // PTYN row
            Constraint::Length(1), // PS row
            Constraint::Min(0),    // remaining space
        ])
        .split(inner_area);

    {
        let rta_label = Paragraph::new("RTA:").style(Style::default().fg(Color::LightCyan));
        let rta = rds_to_utf8_lossy::<RADIOTEXT_LEN>(&rds_data.rt.a.display);
        let rta_content = format!(
            "{:<64}",
            rta.chars().take(MAX_RADIOTEXT_LEN).collect::<String>()
        );
        let rta_input = Paragraph::new(rta_content)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White));
        let rta_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(4),
                Constraint::Length(MAX_RADIOTEXT_LEN as u16),
                Constraint::Min(0),
            ])
            .split(chunks[0]);
        f.render_widget(rta_label, rta_area[0]);
        f.render_widget(rta_input, rta_area[1]);
    }

    {
        let label = Paragraph::new("RTB:").style(Style::default().fg(Color::LightCyan));
        let rtb = rds_to_utf8_lossy::<RADIOTEXT_LEN>(&rds_data.rt.b.display);
        let rtb_content = format!(
            "{:<64}",
            rtb.chars().take(MAX_RADIOTEXT_LEN).collect::<String>()
        );
        let rtb_input = Paragraph::new(rtb_content)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White));
        let rtb_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(4),
                Constraint::Length(MAX_RADIOTEXT_LEN as u16),
                Constraint::Min(0),
            ])
            .split(chunks[1]);
        f.render_widget(label, rtb_area[0]);
        f.render_widget(rtb_input, rtb_area[1]);
    }

    {
        const PTYN_LEN: u8 = 8;
        let label = Paragraph::new("PTYN:").style(Style::default().fg(Color::LightCyan));
        let data = rds_to_utf8_lossy::<PS_LEN>(&rds_data.ptyn.display);
        let text = format!("{:<8}", data.chars().take(8).collect::<String>());
        let input =
            Paragraph::new(text).style(Style::default().bg(Color::DarkGray).fg(Color::White));
        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(5),
                Constraint::Length(PTYN_LEN as u16),
                Constraint::Min(0),
            ])
            .split(chunks[2]);
        f.render_widget(label, area[0]);
        f.render_widget(input, area[1]);
    }

    {
        let label = Paragraph::new("PS:").style(Style::default().fg(Color::LightCyan));
        let data = rds_to_utf8_lossy::<PS_LEN>(&rds_data.tn.ps.display);
        let text = format!("{:<8}", data.chars().take(8).collect::<String>());
        let input =
            Paragraph::new(text).style(Style::default().bg(Color::DarkGray).fg(Color::White));
        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(PS_LEN as u16),
                Constraint::Min(0),
            ])
            .split(chunks[2]);
        f.render_widget(label, area[0]);
        f.render_widget(input, area[1]);
    }
}

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
            draw_ui(f, rds_data, block_idx + 1, rds_blocks.len());
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => {
                    break;
                }
                KeyCode::Left => {
                    if block_idx > 0 {
                        block_idx -= 1;
                    }
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

fn process_reader<R: BufRead + 'static>(reader: R) -> io::Result<Vec<RdsData>> {
    let mut rds_blocks: Vec<RdsData> = Vec::new();

    let mut rds_data = RdsData::default();
    let mut decoder = Decoder::new(false);
    for group_result in RdsGroupIterator::new(reader) {
        match group_result {
            Ok(group) => {
                let blocks = Group {
                    a: group.a,
                    b: group.b,
                    c: group.c,
                    d: group.d,
                };
                decoder.decode(&blocks, &mut rds_data);
                rds_blocks.push(rds_data.clone());
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    Ok(rds_blocks)
}
