use crate::game::{Board, GameCell, Position, State};
use rand::Rng;

#[derive(Debug, Clone, Copy)]
pub enum Opponent {
    Human,
    Random,
    Minimax,
}

pub fn get_pos(player: Opponent, board: &Board, cell: &GameCell) -> Result<Position, String> {
    match player {
        Opponent::Random => random_play(board),
        Opponent::Minimax => minimax_play(board, cell),
        Opponent::Human => Err("Player is not allowed to play".to_string()),
    }
}

fn random_play(board: &Board) -> Result<Position, String> {
    let mut rng = rand::thread_rng();
    let available_moves = board.available_moves();
    if available_moves.len() == 0 {
        return Err("No available moves".to_string());
    }
    let move_index = rng.gen_range(0..available_moves.len());
    Ok(available_moves[move_index])
}

fn minimax_play(board: &Board, cell: &GameCell) -> Result<Position, String> {
    let available_moves = board.available_moves();
    if available_moves.len() == 0 {
        return Err("No available moves".to_string());
    }
    let best_move = minimax(board, cell);
    Ok(best_move.unwrap())
}

fn minimax(board: &Board, cell: &GameCell) -> Result<Position, String> {
    if board.cells[0][0] == GameCell::Empty {
        return Ok(Position { x: 0, y: 0 });
    }
    let mut best_move = None;
    let mut best_score = i64::min_value();
    for m in board.available_moves().iter() {
        let mut new_board = board.clone();
        new_board.set_cell(*m, *cell);
        let score = minimax_score(
            &mut new_board,
            cell,
            3,
            true,
            i64::min_value(),
            i64::max_value(),
            3,
        );
        if score > best_score {
            best_move = Some(*m);
            best_score = score;
        };
    }
    Ok(best_move.unwrap())
}

fn evaluate(board: &Board, cell: &GameCell) -> i64 {
    match board.get_state() {
        State::Win(c) => {
            if c == *cell {
                1000
            } else {
                -1000
            }
        }
        State::Draw => 0,
        _ => 0,
    }
}
fn minimax_score(
    board: &mut Board,
    cell: &GameCell,
    depth: i64,
    is_maximizing: bool,
    mut alpha: i64,
    mut beta: i64,
    max_depth: i64,
) -> i64 {
    let moves = board.available_moves();
    if depth == 0 || board.get_state() != State::Empty || moves.len() == 0 {
        return evaluate(board, cell);
    }

    if is_maximizing {
        let mut value = i64::min_value();
        for idx in moves {
            board.set_cell(idx, *cell);
            let score = minimax_score(
                board,
                &cell.opposite(),
                depth - 1,
                false,
                alpha,
                beta,
                max_depth,
            );
            if score >= value {
                value = score;
            }
            if score >= alpha {
                alpha = score;
            }
            board.set_cell(idx, GameCell::Empty);
            if beta <= alpha {
                break;
            }
        }
        if value != 0 {
            return value - (max_depth - depth) as i64;
        }
        value
    } else {
        let mut value = i64::max_value();
        for idx in moves {
            board.set_cell(idx, cell.opposite());
            let score = minimax_score(
                board,
                &cell.opposite(),
                depth - 1,
                true,
                alpha,
                beta,
                max_depth,
            );
            if score <= value {
                value = score;
            }
            if score <= beta {
                beta = score;
            }
            board.set_cell(idx, GameCell::Empty);
            if beta <= alpha {
                break;
            }
        }

        if value != 0 {
            return value + (max_depth - depth) as i64;
        }
        value
    }
}

// fn minimax_score(board: &Board, cell: &GameCell, mut alpha: i32, beta: i32) -> i32 {
//     match board.get_state() {
//         State::Win(c) => {
//             if c == *cell {
//                 1
//             } else {
//                 -1
//             }
//         }
//         State::Draw => 0,
//         State::Empty => {
//             let mut best_score = i32::min_value();
//             for m in board.available_moves().iter() {
//                 let mut new_board = board.clone();
//                 new_board.set_cell(*m, *cell);
//                 let score = minimax_score(&new_board, cell, alpha, beta);
//                 if score > best_score {
//                     best_score = score;
//                 };
//                 if best_score >= beta {
//                     return best_score;
//                 }
//                 alpha = std::cmp::max(alpha, best_score);
//             }
//             best_score
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let board = Board::new();
        let mov = get_pos(Opponent::Random, &board, &GameCell::Cross);
        assert_eq!(true, board.available_moves().contains(&mov.unwrap()));
    }

    #[test]
    fn test_minimax() {
        let mut board = Board::new();
        let mut cell = GameCell::Cross;
        println!("{:?}", board.cells);
        let mut  i = 0;
        loop {
            assert_eq!(State::Empty, board.get_state());
            println!("{:?}", board.cells);
            let mov = get_pos(Opponent::Minimax, &board, &cell).unwrap();
            assert_eq!(true, board.available_moves().contains(&mov));
            board.set_cell(mov, cell);
            if board.get_state() != State::Empty {
                break;
            }
            cell = cell.opposite();
            i += 1;
            if i == 9 {
                break;
            }
        }
    }
}
