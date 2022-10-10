use serde::{Deserialize, Serialize};
use std::ops::AddAssign;
// pub enum Key {
//     Char(char),
//     Enter,
//     Up,
//     Down,
//     Left,
//     Right,
//     Esc,
//     Unknown,
// }

#[derive(Serialize, Deserialize, Debug)]
pub enum Move {
    Down,
    Up,
    Left,
    Right,
    Place,
    PlaceAt(Position),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Score {
    pub player1: u32,
    pub player2: u32,
}

impl Score {
    pub fn default() -> Score {
        Score { player1: 0, player2: 0 }
    }
}

impl AddAssign for Score {
    fn add_assign(&mut self, other: Score) {
        self.player1 += other.player1;
        self.player2 += other.player2;
    }
}

impl Position {
    pub fn to_tuple(&self) -> (usize, usize) {
        (self.y, self.x)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    Move(Move),
    ToggleMenu,
    Select(u8),
    // Input(String),
    Quit,
    Reset,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Opponent {
    Online,
    Human,
    Random,
    Minimax,
    // Online(OnlinePlayer)
}

#[derive(Clone, PartialEq, Copy, Debug, Serialize, Deserialize)]
pub enum GameCell {
    Empty,
    Cross,
    Circle,
}

impl GameCell {
    pub fn opposite(&self) -> Self {
        match self {
            GameCell::Cross => GameCell::Circle,
            GameCell::Circle => GameCell::Cross,
            _ => GameCell::Empty,
        }
    }
}

impl GameCell {
    pub fn to_text(&self, pos: Option<(usize, usize)>) -> String {
        let centre = match self {
            GameCell::Empty => String::from("L"),
            GameCell::Cross => String::from("X"),
            GameCell::Circle => String::from("O"),
        };
        // TODO: Print proper positions with borders
        match pos {
            Some((x, y)) => match (x, y) {
                (0, 0) => centre,
                (0, 1) => centre,
                (0, 2) => centre,
                (1, 0) => centre,
                (1, 1) => centre,
                (1, 2) => centre,
                (2, 0) => centre,
                (2, 1) => centre,
                (2, 2) => centre,
                _ => panic!("Invalid coordinates: {}, {}", x, y),
            },
            None => centre,
        }
    }
}
