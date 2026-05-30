use std::io;

pub mod app;

use app::App;

fn try_main() -> anyhow::Result<()> {
    let mut terminal = ratatui::init();

    App::default().run(&mut terminal).map_err(anyhow::Error::from)
}

fn main() -> anyhow::Result<()> {

    match try_main() {
        Ok(_) => println!("calendar finished with no errrors"),
        Err(e) =>  eprintln!("calendar finished with errors: {}", e),
    }

    Ok(())
}
