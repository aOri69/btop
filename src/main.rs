use btop::{restore_terminal, run_app, setup_terminal, App, Config, Parser};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = setup_terminal()?;

    let config = Config::parse();
    let app = App::new(config);
    // Main loop
    let result = run_app(&mut terminal, app);

    restore_terminal(terminal)?;

    if let Err(err) = result {
        eprintln!("{err:?}");
    }

    Ok(())
}
