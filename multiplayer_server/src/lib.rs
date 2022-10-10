use serde::{Deserialize, Serialize};
use tictactoe_library::{
    game::{Game, GameState},
    update::{Move, Opponent, Position, Score},
};
// use tokio::sync::mpsc;
// use warp::ws::Message;

// #[derive(Debug, Clone)]
// pub struct Session {
//     pub id: String,
//     pub client_status: HashMap<String, bool>,
//     pub data: ServerGameState,
// }

// #[derive(Debug, Clone)]
// pub struct Client {
//     /// Unique ID of the Client
//     pub id: String,
//     /// Session ID the client belongs to if it exists
//     pub session_id: Option<String>,
//     /// Sender pipe used to relay messages to the sender socket
//     pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
// }

/// Events that a client is supposed to emit to the server
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientEvent {
    ListSessions,
    CreateSession,
    LeaveSession,
    JoinSession(String),
    GameEvent(Move),
}

/// Events that the server emits after a client has sent a message
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ServerEvent {
    /// The list of sessions in the server
    /// Response for [`ClientEvent::ListSessions`]
    ListSessions(Vec<String>),
    /// The `ServerGameState` of the session, containing the game state and the list of Players
    /// Response for `ClientEvent::JoinSession`
    GameStart(ServerGameState),
    /// The updated `ServerApp` state after a move has been made
    /// Response for `ClientEvent::GameEvent`
    /// Auto Response to client
    GameUpdate(ServerApp),
    /// The session has been created
    /// Response for [`ClientEvent::CreateSession`] and [`ClientEvent::JoinSession`]
    /// Contains the created session id
    Queue(String),
    /// Error event from the server
    /// Response for any [`ClientEvent`]
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Players {
    Full(PlayerData, PlayerData, String),
    Partial(PlayerData),
    Empty,
}

// impl Players {
//     pub fn get_client_ids(self) -> Option<Vec<String>> {
//         match self {
//             Players::Full(a, b, _) => Some(vec![a.id, b.id]),
//             Players::Partial(_) => None,
//             Players::Empty => None,
//         }
//     }

// pub fn other(self, client_id: String) -> Option<String> {
//     if let Players::Full(player_1, player_2, _) = self {
//         if player_1.id == client_id {
//             return Some(player_2.id);
//         } else if player_2.id == client_id {
//             return Some(player_1.id);
//         }
//     }
//     None
// }

// pub fn get_mut(self, client_id: String) -> Option<&'static mut PlayerData> {
//     if let Players::Full(player_1, player_2, _) = self {
//         if player_1.id == client_id {
//             return Some(&mut player_2);
//         } else if player_2.id == client_id {
//             return Some(&mut player_1);
//         }
//     } else if let Players::Partial(player) = self {
//         if player.id == client_id {
//             return Some(&mut player);
//         }
//     }
//     None
// }
// }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerApp {
    pub game: Game,
    pub game_state: GameState,
    pub score: Score,
}

impl ServerApp {
    pub fn default() -> Self {
        let game = Game::new(Opponent::Human);
        ServerApp {
            game_state: game.clone().get_state().unwrap(),
            game,
            score: Score::default(),
        }
    }
    pub fn update(&mut self, mv: Move) -> Result<(), String> {
        match self.game_state {
            GameState::GameInProgress(..) => match self.game.update(mv) {
                Ok(state) => {
                    if let GameState::GameInProgress(..) = state {
                        self.score += self.game.get_score();
                    };
                    self.game_state = state;
                    Ok(())
                }
                Err(e) => Err(e),
            },
            GameState::GameOver(..) => Err("Game is not in progress".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerGameState {
    pub players: Players,
    pub game: Option<ServerApp>,
}

impl ServerGameState {
    pub fn default() -> Self {
        Self {
            players: Players::Empty,
            game: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerData {
    pub id: String,
    // pub name: String,
    pub current_pos: Position,
}

impl PlayerData {
    pub fn new(id: &String) -> Self {
        Self {
            id: id.to_string(),
            current_pos: Position::default(),
        }
    }
}
