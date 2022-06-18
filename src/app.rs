use crate::{
    game::{Game, GameState},
    player::Opponent,
    update::Key,
};
use std::ops::AddAssign;

#[derive(Debug, PartialEq, Clone)]
pub enum AppState {
    StartMenu(u8),
    Playing(GameState),
    GameMenu(u8),
    Quit,
}

pub struct App {
    game: Game,
    pub score: Score,
    pub state: AppState,
    pub warning_message: Option<String>,
    pub prev_state: Option<GameState>,
}

pub struct Score {
    pub player1: u32,
    pub player2: u32,
}

impl Score {
    pub fn new() -> Score {
        Score {
            player1: 0,
            player2: 0,
        }
    }
}

impl AddAssign for Score {
    fn add_assign(&mut self, other: Score) -> () {
        self.player1 += other.player1;
        self.player2 += other.player2;
    }
}

impl App {
    pub fn new() -> App {
        let game = Game::new(Opponent::Human);
        App {
            game,
            score: Score::new(),
            state: AppState::StartMenu(0),
            warning_message: None,
            prev_state: None,
        }
    }

    pub fn quit(&mut self) {
        self.state = AppState::Quit;
    }

    fn toggle_menu(&mut self) {
        match &self.state {
            AppState::GameMenu(_) => {
                self.state = AppState::Playing(self.prev_state.clone().unwrap())
            }
            AppState::Playing(state) => {
                self.prev_state = Some(state.clone());
                self.state = AppState::GameMenu(0);
            }
            _ => (),
        }
    }

    fn next_row_menu(&mut self, up: bool) {
        match self.state {
            AppState::GameMenu(ref mut row) => {
                if up {
                    *row = (*row + 1) % 3;
                } else {
                    *row = (*row + 2) % 3;
                }
            }
            AppState::StartMenu(ref mut row) => {
                if up {
                    *row = (*row + 1) % 3;
                } else {
                    *row = (*row + 2) % 3;
                }
            }
            _ => {}
        }
    }
    fn start_game(&mut self, opponent: Opponent) {
        let game = Game::new(opponent);
        self.game = game;
        self.state = AppState::Playing(self.game.get_state().unwrap());
    }

    pub fn update(&mut self, key: Key) {
        match self.state {
            AppState::GameMenu(x) => match key {
                Key::Char(c) => match c {
                    'q' => self.quit(),
                    'm' => self.toggle_menu(),
                    _ => {}
                },
                Key::Enter => match x {
                    0 => self.toggle_menu(),
                    1 => self.reset(),
                    2 => self.quit(),
                    _ => self.state = AppState::GameMenu(0),
                },
                Key::Up => self.next_row_menu(false),
                Key::Down => self.next_row_menu(true),
                _ => {}
            },

            AppState::Playing(_) => {
                match key {
                    Key::Char(c) => self.on_key(c),
                    Key::Esc => self.toggle_menu(),
                    Key::Up => self.game.on_up(),
                    Key::Down => self.game.on_down(),
                    Key::Left => self.game.on_left(),
                    Key::Right => self.game.on_right(),
                    _ => {}
                }
                if let Some(state) = self.game.get_state() {
                    match state {
                        GameState::GameOver(..) => self.score += self.game.get_score(),
                        _ => {}
                    }
                    self.state = AppState::Playing(state);
                    self.warning_message = self.game.get_warning_message();
                }
            }

            AppState::StartMenu(x) => match key {
                Key::Char(c) => match c {
                    'q' => self.quit(),
                    _ => {}
                },
                Key::Enter => match x {
                    0 => self.start_game(Opponent::Human),
                    1 => self.start_game(Opponent::Random),
                    2 => self.start_game(Opponent::Minimax),
                    _ => self.state = AppState::StartMenu(0),
                },
                Key::Up => self.next_row_menu(false),
                Key::Down => self.next_row_menu(true),
                _ => {}
            },
            _ => {}
        };
    }

    fn reset(&mut self) {
        self.game = Game::new(self.game.opponent.clone());
        self.state = AppState::Playing(self.game.get_state().unwrap());
        self.warning_message = self.game.get_warning_message();
    }

    pub fn on_key(&mut self, char: char) {
        match char {
            'p' => {
                if !self.game.is_over() {
                    self.game.place()
                }
            }
            'q' => self.quit(),
            'r' => self.reset(),
            'm' => self.toggle_menu(),
            _ => {
                if self.game.is_over() {
                    self.game.warning_message = None;
                }
            }
        };
    }
}

// tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_menu() {
        let mut app = App::new("test".to_string());
        assert_eq!(app.state, AppState::StartMenu(0));
        app.next_row_menu(true);
        assert_eq!(app.state, AppState::StartMenu(1));
        app.next_row_menu(true);
        assert_eq!(app.state, AppState::StartMenu(2));
        app.next_row_menu(true);
        assert_eq!(app.state, AppState::StartMenu(0));
    }
}
