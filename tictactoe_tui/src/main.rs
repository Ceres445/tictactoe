mod crossterm;
mod ui;

use crate::crossterm::run;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    run()?;
    Ok(())
}
