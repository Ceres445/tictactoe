use crate::{
    game::{Game, GameState},
    player::Opponent,
    update::{Action, Move},
};
use std::ops::AddAssign;

#[derive(Debug, PartialEq, Clone)]
pub enum AppState {
    Menu(Menu, u8),
    Playing(GameState),
    Quit,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Menu {
    Start,
    Game,
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
            state: AppState::Menu(Menu::Start, 0),
            warning_message: None,
            prev_state: None,
        }
    }

    pub fn quit(&mut self) {
        self.state = AppState::Quit;
    }

    fn toggle_menu(&mut self) -> Result<(), String> {
        match &self.state {
            AppState::Menu(menu, _) => {
                match menu {
                    Menu::Start => return Err("Cannot go back from game menu".to_string()),
                    Menu::Game => self.state = AppState::Playing(self.prev_state.clone().unwrap()),
                }
                Ok(())
            }
            AppState::Playing(state) => {
                self.prev_state = Some(state.clone());
                self.state = AppState::Menu(Menu::Game, 0);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn next_row_menu(&mut self, up: bool) {
        match self.state {
            AppState::Menu(_, ref mut row) => {
                if up {
                    *row = (*row + 1) % 3;
                } else {
                    *row = (*row + 2) % 3;
                }
            }
            _ => self.warning_message = Some("Cannot change menu row in this state".to_string()),
        }
    }
    fn start_game(&mut self, opponent: Opponent) {
        let game = Game::new(opponent);
        self.game = game;
        self.state = AppState::Playing(self.game.get_state().unwrap());
    }

    pub fn update(&mut self, action: Action) {
        match action {
            Action::Move(mv) => match &self.state {
                AppState::Playing(state) => match state {
                    GameState::GameInProgress(..) => match self.game.update(mv) {
                        Ok(state) => {
                            match state {
                                GameState::GameOver(..) => self.score += self.game.get_score(),
                                _ => {}
                            };
                            self.state = AppState::Playing(state);
                        }
                        Err(message) => self.warning_message = Some(message),
                    },
                    _ => self.warning_message = Some("Cannot move in this state".to_string()),
                },
                AppState::Menu(menu, row) => match mv {
                    Move::Down => self.next_row_menu(true),
                    Move::Up => self.next_row_menu(false),
                    Move::Place => match menu {
                        Menu::Start => match row {
                            0 => self.start_game(Opponent::Human),
                            1 => self.start_game(Opponent::Random),
                            2 => self.start_game(Opponent::Minimax),
                            _ => self.state = AppState::Menu(Menu::Start, 0),
                        },
                        Menu::Game => match row {
                            0 => self.toggle_menu().unwrap(),
                            1 => self.reset(),
                            2 => self.quit(),
                            _ => self.state = AppState::Menu(Menu::Game, 0),
                        },
                    },
                    _ => self.warning_message = Some("Cannot move in this state".to_string()),
                },
                _ => self.warning_message = Some("Cannot move in this state".to_string()),
            },
            Action::Select(row) => match &self.state {
                AppState::Menu(menu, _) => match menu {
                    Menu::Start => self.start_game(match row {
                        0 => Opponent::Human,
                        1 => Opponent::Random,
                        2 => Opponent::Minimax,
                        _ => Opponent::Human,
                    }),
                    Menu::Game => match row {
                        0 => self.quit(),
                        1 => self.toggle_menu().unwrap(),
                        _ => self.warning_message = Some("Invalid menu row".to_string()),
                    },
                },
                _ => self.warning_message = Some("Cannot select in this state".to_string()),
            },
            Action::Quit => {
                self.quit();
            }
            Action::Reset => {
                self.reset();
            }
            Action::ToggleMenu => {
                self.toggle_menu().unwrap();
            }
        }
    }

    fn reset(&mut self) {
        self.game = Game::new(self.game.opponent.clone());
        self.state = AppState::Playing(self.game.get_state().unwrap());
    }
}

// tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_menu() {
        let mut app = App::new();
        assert_eq!(app.state, AppState::Menu(Menu::Start, 0));
        app.next_row_menu(true);
        assert_eq!(app.state, AppState::Menu(Menu::Start, 1));
        app.next_row_menu(true);
        assert_eq!(app.state, AppState::Menu(Menu::Start, 2));
        app.next_row_menu(true);
        assert_eq!(app.state, AppState::Menu(Menu::Start, 0));
    }
}

// pub fn update(&mut self, key: Key) {
//     match self.state {
//         AppState::GameMenu(x) => match key {
//             Key::Char(c) => match c {
//                 'q' => self.quit(),
//                 'm' => self.toggle_menu(),
//                 _ => {}
//             },
//             Key::Enter => match x {
//                 0 => self.toggle_menu(),
//                 1 => self.reset(),
//                 2 => self.quit(),
//                 _ => self.state = AppState::GameMenu(0),
//             },
//             Key::Up => self.next_row_menu(false),
//             Key::Down => self.next_row_menu(true),
//             _ => {}
//         },

//         AppState::Playing(_) => {
//             match key {
//                 Key::Char(c) => self.on_key(c),
//                 Key::Esc => self.toggle_menu(),
//                 Key::Up => self.game.on_up(),
//                 Key::Down => self.game.on_down(),
//                 Key::Left => self.game.on_left(),
//                 Key::Right => self.game.on_right(),
//                 _ => {}
//             }
// if let Some(state) = self.game.get_state() {
//     match state {
//         GameState::GameOver(..) => self.score += self.game.get_score(),
//         _ => {}
//     }
//     self.state = AppState::Playing(state);
//     self.warning_message = self.game.get_warning_message();
// }
//         }

//         AppState::Menu(Menu::Start, x) => match key {
//             Key::Char(c) => match c {
//                 'q' => self.quit(),
//                 _ => {}
//             },
//             Key::Enter => match x {
//                 0 => self.start_game(Opponent::Human),
//                 1 => self.start_game(Opponent::Random),
//                 2 => self.start_game(Opponent::Minimax),
//                 _ => self.state = AppState::Menu(Menu::Start, 0),
//             },
//             Key::Up => self.next_row_menu(false),
//             Key::Down => self.next_row_menu(true),
//             _ => {}
//         },
//         _ => {}
//     };
// }

// pub fn on_key(&mut self, char: char) {
//     match char {
//         'p' => {
//             if !self.game.is_over() {
//                 self.game.place()
//             }
//         }
//         'q' => self.quit(),
//         'r' => self.reset(),
//         'm' => self.toggle_menu(),
//         _ => {
//             if self.game.is_over() {
//                 self.game.warning_message = None;
//             }
//         }
//     };
// }
