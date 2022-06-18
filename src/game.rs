use core::fmt;

use adorn::adorn_method;

use crate::{
    app::Score,
    player::{get_pos, Opponent},
};

#[derive(Clone, PartialEq, Copy, Debug)]
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
                (0, 0) => format!("{}", centre),
                (0, 1) => format!("{}", centre),
                (0, 2) => format!("{}", centre),
                (1, 0) => format!("{}", centre),
                (1, 1) => format!("{}", centre),
                (1, 2) => format!("{}", centre),
                (2, 0) => format!("{}", centre),
                (2, 1) => format!("{}", centre),
                (2, 2) => format!("{}", centre),
                _ => panic!("Invalid coordinates: {}, {}", x, y),
            },
            None => centre,
        }
    }
}

pub type Cells = Vec<Vec<GameCell>>;

#[derive(Clone)]
pub struct Board {
    pub cells: Cells,
}

impl Board {
    pub fn new() -> Board {
        Board {
            cells: vec![vec![GameCell::Empty; 3]; 3],
        }
    }
    pub fn get_cell(&self, pos: Position) -> Option<&GameCell> {
        Some(self.cells.get(pos.y)?.get(pos.x)?)
    }

    pub fn set_cell(&mut self, pos: Position, cell: GameCell) {
        self.cells[pos.y][pos.x] = cell;
    }

    pub fn available_moves(&self) -> Vec<Position> {
        let mut moves = Vec::new();
        for y in 0..3 {
            for x in 0..3 {
                if self.cells[y][x] == GameCell::Empty {
                    moves.push(Position { y, x });
                }
            }
        }
        moves
    }

    pub fn moves(&self) -> usize {
        let mut moves = 0;
        for k in 0..3 {
            for j in 0..3 {
                if self.cells[k][j] != GameCell::Empty {
                    moves += 1;
                }
            }
        }
        moves
    }

    pub fn get_state(&self) -> State {
        let check = |x: GameCell, y: GameCell, z: GameCell| {
            if x == y && y == z && x != GameCell::Empty {
                Some(x)
            } else {
                None
            }
        };
        let cells = self.cells.clone();
        let rows = self
            .cells
            .iter()
            .map(|row| check(row[0], row[1], row[2]))
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect::<Vec<_>>();
        let cols = (0..3)
            .map(|col| check(cells[0][col], cells[1][col], cells[2][col]))
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect::<Vec<_>>();
        let diag1 = check(cells[0][0], cells[1][1], cells[2][2]).unwrap_or(GameCell::Empty);
        let diag2 = check(cells[0][2], cells[1][1], cells[2][0]).unwrap_or(GameCell::Empty);

        let mut state = State::Empty;
        if rows.len() == 1 {
            state = State::Win(rows[0]);
        } else if cols.len() == 1 {
            state = State::Win(cols[0]);
        } else if diag1 != GameCell::Empty {
            state = State::Win(diag1);
        } else if diag2 != GameCell::Empty {
            state = State::Win(diag2);
        } else if self.available_moves().is_empty() {
            state = State::Draw;
        }

        state
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State {
    Empty,
    Win(GameCell),
    Draw,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GameState {
    GameOver(Option<Player>, Cells),
    GameInProgress(Cells, Player, Position),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn to_tuple(&self) -> (usize, usize) {
        (self.y, self.x)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Player {
    Player1,
    Player2,
}

impl Player {
    pub fn next(&self) -> Player {
        match self {
            Player::Player1 => Player::Player2,
            Player::Player2 => Player::Player1,
        }
    }

    pub fn get_cell(&self) -> GameCell {
        match self {
            Player::Player1 => GameCell::Cross,
            Player::Player2 => GameCell::Circle,
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Player::Player1 => write!(f, "Player 1 (X)"),
            Player::Player2 => write!(f, "Player 2 (O)"),
        }
    }
}

pub struct Game {
    pub board: Board,
    pub current_position: Position,
    pub current_player: Player,
    pub winner: Option<Player>,
    pub warning_message: Option<String>,
    pub opponent: Opponent,
    should_continue: bool,
    state_changed: bool,
}

impl Game {
    pub fn new(opponent: Opponent) -> Game {
        Game {
            board: Board::new(),
            current_position: Position { x: 0, y: 0 },
            current_player: Player::Player1,
            winner: None,
            opponent,
            should_continue: true,
            warning_message: None,
            state_changed: true,
        }
    }

    fn get_current_player_cell(&self) -> GameCell {
        match self.current_player {
            Player::Player1 => GameCell::Cross,
            Player::Player2 => GameCell::Circle,
        }
    }

    fn opponent_move(&mut self) {
        let pos = match get_pos(
            self.opponent.clone(),
            &self.board,
            &self.current_player.get_cell(),
        ) {
            Ok(pos) => pos,
            Err(e) => {
                self.warning_message = Some(e.to_string());
                return;
            }
        };
        self.board.set_cell(pos, self.current_player.get_cell());
        self.next();
    }

    fn next(&mut self) {
        match self.board.get_state() {
            State::Empty => {
                self.current_player = self.current_player.next();
                self.state_changed = true;
                if let Opponent::Human = self.opponent {
                    return;
                }
                if let Player::Player2 = self.current_player {
                    self.opponent_move();
                }
            }
            State::Win(_) => {
                self.winner = Some(self.current_player);
                self.should_continue = false;
            }
            State::Draw => {
                self.winner = None;
                self.should_continue = false;
            }
        }
    }
    pub fn is_over(&self) -> bool {
        self.should_continue == false
    }

    pub fn get_score(&self) -> Score {
        if self.winner.is_some() {
            match self.winner.unwrap() {
                Player::Player1 => Score {
                    player1: 1,
                    player2: 0,
                },
                Player::Player2 => Score {
                    player1: 0,
                    player2: 1,
                },
            }
        } else {
            Score::new()
        }
    }
    pub fn get_warning_message(&self) -> Option<String> {
        self.warning_message.clone()
    }
    pub fn place(&mut self) {
        self.state_changed = true;
        self.warning_message = None;
        if let Some(cell) = self.board.get_cell(self.current_position.clone()) {
            match cell {
                GameCell::Empty => {
                    self.board.set_cell(
                        self.current_position.clone(),
                        self.get_current_player_cell(),
                    );
                    self.next();
                }
                _ => {
                    self.warning_message = Some("This cell is already taken!".to_string());
                    return;
                }
            }
        } else {
            self.warning_message = Some("This cell is out of bounds!".to_string());
        }
    }

    pub fn get_state(&mut self) -> Option<GameState> {
        if self.state_changed {
            self.state_changed = false;
            if self.is_over() {
                Some(GameState::GameOver(self.winner, self.board.cells.clone()))
            } else {
                Some(GameState::GameInProgress(
                    self.board.cells.clone(),
                    self.current_player,
                    self.current_position.clone(),
                ))
            }
        } else {
            None
        }
    }

    fn wrap<F>(&mut self, func: F)
    where
        F: Fn(&mut Self),
    {
        if self.should_continue {
            self.state_changed = true;
            func(self);
        } else {
            self.warning_message = Some("Game is over!".to_string());
        }
    }

    #[adorn_method(wrap)]
    pub fn on_up(&mut self) {
        self.warning_message = None;
        if self.current_position.y > 0 {
            self.current_position.y -= 1;
        }
    }

    #[adorn_method(wrap)]
    pub fn on_down(&mut self) {
        self.warning_message = None;
        if self.current_position.y < 2 {
            self.current_position.y += 1;
        }
    }

    #[adorn_method(wrap)]
    pub fn on_left(&mut self) {
        self.warning_message = None;
        if self.current_position.x > 0 {
            self.current_position.x -= 1;
        }
    }

    #[adorn_method(wrap)]
    pub fn on_right(&mut self) {
        self.warning_message = None;
        if self.current_position.x < 2 {
            self.current_position.x += 1;
        }
    }
}

// write tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_up() {
        let mut game = Game::new(Opponent::Human);
        game.current_position = Position { x: 0, y: 0 };
        game.on_down();
        assert_eq!(game.current_position, Position { x: 0, y: 1 });
    }

    #[test]
    fn test_down() {
        let mut game = Game::new(Opponent::Human);
        game.current_position = Position { x: 0, y: 2 };
        game.on_up();
        assert_eq!(game.current_position, Position { x: 0, y: 1 });
    }

    #[test]
    fn test_left() {
        let mut game = Game::new(Opponent::Human);
        game.current_position = Position { x: 0, y: 0 };
        game.on_right();
        assert_eq!(game.current_position, Position { x: 1, y: 0 });
    }

    #[test]
    fn test_right() {
        let mut game = Game::new(Opponent::Human);
        game.current_position = Position { x: 2, y: 0 };
        game.on_left();
        assert_eq!(game.current_position, Position { x: 1, y: 0 });
    }

    #[test]
    fn test_place() {
        let mut game = Game::new(Opponent::Human);
        game.current_position = Position { x: 0, y: 0 };
        assert_eq!(game.current_player, Player::Player1);
        game.place();
        assert_eq!(
            game.board.get_cell(Position { x: 0, y: 0 }),
            Some(&GameCell::Cross)
        );
        assert_eq!(game.current_player, Player::Player2);
        assert_eq!(game.current_position, Position { x: 0, y: 0 });
        println!("{:?}", game.board.cells);

        game.on_right();
        game.place();
        println!("{:?}", game.board.cells);
        assert_eq!(game.board.get_state(), State::Empty);
        assert_eq!(game.current_player, Player::Player1);
        assert_eq!(game.current_position, Position { x: 1, y: 0 });
        assert_eq!(
            game.board.get_cell(Position { x: 1, y: 0 }),
            Some(&GameCell::Circle)
        );
    }

    #[test]
    fn test_minimax() {
        let mut game = Game::new(Opponent::Minimax);
        game.current_position = Position { x: 0, y: 0 };
        game.place();
        assert_eq!(
            game.board.get_cell(Position { x: 0, y: 0 }),
            Some(&GameCell::Cross)
        );
        assert_eq!(game.current_player, Player::Player1);
        assert_eq!(game.current_position, Position { x: 0, y: 0 });
    }

    #[test]
    fn test_random() {
        let mut game = Game::new(Opponent::Random);
        game.current_position = Position { x: 0, y: 0 };
        game.place();
        assert_eq!(
            game.board.get_cell(Position { x: 0, y: 0 }),
            Some(&GameCell::Cross)
        );
        assert_eq!(game.current_player, Player::Player1);
        assert_eq!(game.current_position, Position { x: 0, y: 0 });
        assert_eq!(game.board.moves(), 2);
        assert_eq!(game.board.get_state(), State::Empty)
    }

    #[test]
    fn test_available_moves() {
        let mut game = Game::new(Opponent::Human);
        game.current_position = Position { x: 0, y: 0 };
        game.board.cells = vec![
            vec![GameCell::Cross, GameCell::Empty, GameCell::Empty],
            vec![GameCell::Cross, GameCell::Empty, GameCell::Empty],
            vec![GameCell::Cross, GameCell::Empty, GameCell::Empty],
        ];
        assert_eq!(game.board.moves(), 3);
        assert_eq!(
            game.board
                .available_moves()
                .contains(&Position { x: 1, y: 0 }),
            true
        );
        assert_eq!(
            game.board
                .available_moves()
                .contains(&Position { x: 1, y: 1 }),
            true
        );
        assert_eq!(
            game.board
                .available_moves()
                .contains(&Position { x: 1, y: 2 }),
            true
        );
        assert_eq!(
            game.board
                .available_moves()
                .contains(&Position { x: 2, y: 0 }),
            true
        );
        assert_eq!(
            game.board
                .available_moves()
                .contains(&Position { x: 2, y: 1 }),
            true
        );
        assert_eq!(
            game.board
                .available_moves()
                .contains(&Position { x: 2, y: 2 }),
            true
        );
        assert_eq!(
            game.board
                .available_moves()
                .contains(&Position { x: 0, y: 1 }),
            false
        );
        assert_eq!(
            game.board
                .available_moves()
                .contains(&Position { x: 0, y: 0 }),
            false
        );
        assert_eq!(
            game.board
                .available_moves()
                .contains(&Position { x: 0, y: 2 }),
            false
        );
    }
}
