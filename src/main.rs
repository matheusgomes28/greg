use std::{fs::File, io::{self, BufReader}};

pub mod app;
pub mod models;
pub mod views;
pub mod ics_utils;

use app::App;
use clap::Parser;
use ical::parser::ical::component::IcalEvent;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct ProgramArgs {
    #[arg(short, long)]
    ics_dir: Option<String>,
}

fn print_event(event: &IcalEvent) {
    for prop in &event.properties {
        println!("{}: {:?}", prop.name, prop.value);
    }
    println!();
}

fn try_main() -> anyhow::Result<()> {
    let args = ProgramArgs::parse();
    if let Some(ics_dir) = args.ics_dir {
        // TODO: Read the directory
        let buf = BufReader::new(File::open(ics_dir)?);
        let reader= ical::IcalParser::new(buf);
        for entry in reader {
            let calendar = entry?;
            for event in calendar.events {
                print_event(&event);
            }
        }

        return Ok(())
    }
    let mut terminal = ratatui::init();

    App::default()
        .run(&mut terminal)
        .map_err(anyhow::Error::from)
}

fn main() -> anyhow::Result<()> {
    let result = try_main();
    ratatui::restore();

    match result {
        Ok(_) => println!("calendar finished with no errrors"),
        Err(e) => eprintln!("calendar finished with errors: {}", e),
    }

    Ok(())
}
