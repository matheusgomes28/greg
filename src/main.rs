pub mod app;
pub mod models;
pub mod views;
pub mod ics_utils;

use app::App;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct ProgramArgs {
    #[arg(short, long)]
    ics_dir: Option<String>,
}

fn try_main() -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    let args = ProgramArgs::parse();
    if let Some(ics_dir) = args.ics_dir {
        let mut app = App::default()
            .with_ics_dir(Some(ics_dir))?;

        return app.run(&mut terminal)
            .map_err(anyhow::Error::from);
    }

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
