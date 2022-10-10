use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tictactoe_library::{
    app::{App, AppState},
    update::{Action, Move},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crate::ui;

pub fn run() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::default();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);
    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                let key = match key.code {
                    event::KeyCode::Char(c) => match c {
                        'q' => Action::Quit,
                        'r' => Action::Reset,
                        'p' => Action::Move(Move::Place),
                        'm' => Action::ToggleMenu,
                        _ => continue,
                    },
                    event::KeyCode::Esc => Action::ToggleMenu,
                    event::KeyCode::Enter => Action::Move(Move::Place),
                    event::KeyCode::Down => Action::Move(Move::Down),
                    event::KeyCode::Up => Action::Move(Move::Up),
                    event::KeyCode::Left => Action::Move(Move::Left),
                    event::KeyCode::Right => Action::Move(Move::Right),
                    _ => continue,
                };
                app.update(key);
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
        if let AppState::Quit = app.state {
            return Ok(());
        }
    }
}
