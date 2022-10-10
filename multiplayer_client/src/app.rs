use multiplayer_server::{ClientEvent, ServerEvent, ServerGameState};
use tictactoe_library::{
    game::{Game, GameState},
    update::{Action, Move, Opponent, Score},
};

pub enum OnlineState {
    Menu,
    // string is session id
    Queue(String),
    Playing(ServerGameState),
}

pub struct ClientWithState {
    pub client: Client,
    pub state: OnlineState,
}

impl ClientWithState {
    pub fn new(name: String) -> Self {
        Self {
            client: Client::new(name),
            state: OnlineState::Menu,
        }
    }

    pub fn change_state(&mut self, state: OnlineState) {
        self.state = state
    }

}

use crate::client::Client;
pub enum AppState {
    Menu(Menu, u8),
    Playing(GameState),
    Online(ClientWithState),
    Quit,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        match self {
            AppState::Menu(menu, u) => AppState::Menu(menu.clone(), *u),
            AppState::Playing(game_state) => AppState::Playing(game_state.clone()),
            AppState::Online(_) => panic!("Cannot clone client"),
            AppState::Quit => AppState::Quit,
            // AppState::PlayingOnline(server_game_state) => AppState::PlayingOnline(server_game_state.clone()),
            // AppState::QueueOnline(session) => AppState::QueueOnline(session.clone()),
        }
    }
}

impl PartialEq for AppState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AppState::Menu(menu, u), AppState::Menu(menu2, u2)) => menu == menu2 && u == u2,
            (AppState::Playing(game_state), AppState::Playing(game_state2)) => game_state == game_state2,
            (AppState::Online(_), AppState::Online(_)) => true,
            (AppState::Quit, AppState::Quit) => true,
            // (AppState::PlayingOnline(server_game_state), AppState::PlayingOnline(server_game_state2)) => {
            //     server_game_state == server_game_state2
            // }
            // (AppState::QueueOnline(session), AppState::QueueOnline(session2)) => session == session2,
            _ => false,
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum Menu {
    Start,
    Game,
}

#[derive(PartialEq, Clone)]
pub struct App {
    game: Game,
    pub score: Score,
    pub state: AppState,
    pub warning_message: Option<String>,
    pub prev_state: Option<GameState>,
}

impl App {
    pub fn default() -> App {
        let game = Game::new(Opponent::Human);
        App {
            game,
            score: Score::default(),
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
        self.game = Game::new(opponent);
        if let Opponent::Online = opponent {
            self.state = AppState::Online(ClientWithState::new("test".to_string()));
            // TODO: Add client name
        } else {
            self.state = AppState::Playing(self.game.get_state().unwrap());
        }
    }

    pub async fn get_lobby(&mut self) -> Result<Vec<String>, String> {
        if let AppState::Online(client_with_state) = &mut self.state {
            if let OnlineState::Menu = client_with_state.state {
                let msg = client_with_state.client.send(ClientEvent::ListSessions).await?;
                if let ServerEvent::ListSessions(sessions) = msg {
                    return Ok(sessions);
                } else {
                    Err("Did not receive lobby".to_string())
                }
            } else {
                Err("Cannot get lobby in this state".to_string())
            }
        } else {
            Err("Not in online mode".to_string())
        }
    }

    pub async fn join_session(&mut self, session: String) -> Result<(), String> {
        if let AppState::Online(client_with_state) = &mut self.state {
            if let OnlineState::Menu = client_with_state.state {
                let msg = client_with_state.client.send(ClientEvent::JoinSession(session)).await?;
                if let ServerEvent::GameStart(game_state) = msg {
                    client_with_state.change_state(OnlineState::Playing(game_state));
                    return Ok(());
                } else {
                    Err("Did not receive lobby".to_string())
                }
            } else {
                Err("Cannot get lobby in this state".to_string())
            }
        } else {
            Err("Not in online mode".to_string())
        }
    }

    pub async fn create_session(&mut self) -> Result<(), String> {
        if let AppState::Online(client) = &mut self.state {
            let msg = client.client.send(ClientEvent::CreateSession).await?;
            if let ServerEvent::Queue(session_id) = msg {
                client.change_state(OnlineState::Queue(session_id));
                return Ok(());
            } else {
                Err("Did not receive lobby".to_string())
            }
        } else {
            Err("Not in online mode".to_string())
        }
    }

    pub async fn update(&mut self, action: Action) {
        self.warning_message = None;
        match action {
            Action::Move(mv) => match &self.state {
                AppState::Playing(state) => match state {
                    GameState::GameInProgress(..) => match self.game.update(mv) {
                        Ok(state) => {
                            if let GameState::GameOver(..) = state {
                                self.score += self.game.get_score();
                            }
                            self.state = AppState::Playing(state);
                        }
                        Err(message) => self.warning_message = Some(message),
                    },
                    GameState::GameOver(..) => self.warning_message = Some("Game is over".to_string()),
                },
                AppState::Menu(menu, row) => match mv {
                    Move::Down => self.next_row_menu(true),
                    Move::Up => self.next_row_menu(false),
                    Move::Place => match menu {
                        Menu::Start => match row {
                            0 => self.start_game(Opponent::Human),
                            1 => self.start_game(Opponent::Random),
                            2 => self.start_game(Opponent::Minimax),
                            3 => self.start_game(Opponent::Online),
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
                        3 => Opponent::Online,
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
            // Action::Input(text) => {

            // }
        }
    }

    fn reset(&mut self) {
        self.game = Game::new(self.game.opponent);
        self.state = AppState::Playing(self.game.get_state().unwrap());
    }
}

mod tests {
    // use super::*;
    // #[test]
    // fn test_start_menu() {
    //     let mut app = App::default();
    //     assert_eq!(app.state, AppState::Menu(Menu::Start, 0));
    //     app.next_row_menu(true);
    //     assert_eq!(app.state, AppState::Menu(Menu::Start, 1));
    //     app.next_row_menu(true);
    //     assert_eq!(app.state, AppState::Menu(Menu::Start, 2));
    //     app.next_row_menu(true);
    //     assert_eq!(app.state, AppState::Menu(Menu::Start, 0));
    // }

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
}
