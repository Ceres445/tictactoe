use tictactoe_library::{
    app::{App, AppState, Menu},
    game::{Cells, GameState, Player},
    update::{Position, GameCell},
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let rects = Layout::default()
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
        .split(f.size());
    let main = Layout::default()
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .direction(Direction::Horizontal)
        .split(rects[0]);
    let state = &app.state;
    match state {
        AppState::Menu(menu, row) => match menu {
            Menu::Start => draw_start_menu(f, *row as usize),
            Menu::Game => {
                let mut menu = Layout::default()
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .direction(Direction::Vertical)
                    .split(main[0]);
                draw_game_menu(f, &mut menu[0], *row);
            }
        },
        AppState::Playing(game_state) => {
            match game_state {
                GameState::GameInProgress(cells, _, pos) => {
                    draw_board(f, cells.to_vec(), pos.clone(), &main[0]);
                }
                GameState::GameOver(winner, cells) => {
                    draw_game_over(f, &main[0], *winner, cells.clone());
                }
            }
            match &app.warning_message {
                Some(message) => draw_warning(f, &rects[1], message.to_string()),
                None => draw_info(f, &rects[1], game_state),
            }

            draw_score(f, app, &main[1], game_state);
        }
        _ => {}
    }
}

fn draw_start_menu<B: Backend>(f: &mut Frame<B>, row: usize) {
    let rect = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .direction(Direction::Vertical)
        .split(f.size());
    let menu = Table::new([
        Row::new([Cell::from("Play against human")]),
        Row::new([Cell::from("Play against random computer")]),
        Row::new([Cell::from("Play against smart computer")]),
    ])
    .block(Block::default().borders(Borders::ALL).title("Start Menu"))
    .highlight_style(Style::default().fg(Color::Yellow))
    .highlight_symbol(">>")
    .widths([Constraint::Percentage(100)].as_ref());

    let mut table_state = TableState::default();
    table_state.select(Some(row));
    f.render_stateful_widget(menu, rect[0], &mut table_state)
}

fn draw_game_menu<B: Backend>(f: &mut Frame<B>, rect: &Rect, row: u8) {
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let table = Table::new([
        Row::new([Cell::from("Resume Game")]),
        Row::new([Cell::from("New Game")]),
        Row::new([Cell::from("Quit")]),
    ])
    .block(Block::default().borders(Borders::ALL).title("Menu"))
    .widths(&[Constraint::Percentage(100)])
    .highlight_style(selected_style)
    .highlight_symbol(">>");

    let mut state = TableState::default();
    state.select(Some(row.into()));
    f.render_stateful_widget(table, *rect, &mut state)
}

fn get_color(player: Player) -> Color {
    match player {
        Player::Player1 => Color::Red,
        Player::Player2 => Color::Blue,
    }
}

fn draw_score<B: Backend>(f: &mut Frame<B>, app: &App, rect: &Rect, game_state: &GameState) {
    let table = Table::new(vec![
        Row::new(vec![Cell::from("Score:".to_string())]),
        Row::new(vec![Cell::from(format!(
            "Player 1's score: {}\nPlayer 2's score: {}",
            app.score.player1, app.score.player2
        ))])
        .style(Style::default().fg(Color::Yellow))
        .height(2),
        if let GameState::GameInProgress(_, player, _) = game_state {
            Row::new(vec![Cell::from(format!("{}'s turn", player,))]).style(Style::default().fg(get_color(*player)))
        } else {
            Row::new(vec![Cell::from("Game Over".to_string())]).style(Style::default().fg(Color::Red))
        },
    ])
    .block(Block::default().borders(Borders::ALL))
    .widths(&[Constraint::Percentage(100)]);

    f.render_widget(table, *rect)
}

fn draw_game_over<B: Backend>(f: &mut Frame<B>, rect: &Rect, winner: Option<Player>, cells: Cells) {
    let mut rows = cells
        .iter()
        .map(|item| {
            let cells = item.iter().map(|c| Cell::from(Span::raw(c.to_text(None))));
            Row::new(cells).height(rect.height / 4)
        })
        .collect::<Vec<_>>();
    let winning_message = match winner {
        Some(winner) => format!("{} wins!", winner),
        None => "It's a draw!".to_string(),
    };
    rows.push(Row::new([Cell::from(Span::raw(&winning_message))]));
    let t = Table::new(rows)
        .widths(&[Constraint::Ratio(1, 3); 3])
        .block(Block::default().title("Game Over").borders(Borders::ALL))
        .style(
            Style::default()
                .fg(if let Some(player) = winner {
                    get_color(player)
                } else {
                    Color::Gray
                })
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(t, *rect);
    // let block = Paragraph::new(format!("Game over! \n{}", winning_message))
    //     .block(Block::default().title("Game Over").borders(Borders::ALL))
    //     .style(
    //         Style::default()
    //             .fg(if let Some(player) = winner {
    //                 player.color()
    //             } else {
    //                 Color::Gray
    //             })
    //             .add_modifier(Modifier::BOLD),
    //     );
    // f.render_widget(block, *rect);
}

fn draw_warning<B: Backend>(f: &mut Frame<B>, rect: &Rect, message: String) {
    let block = Paragraph::new(message)
        .block(Block::default().title("Warning").borders(Borders::ALL))
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
    f.render_widget(block, *rect);
}

fn draw_board<B: Backend>(f: &mut Frame<B>, cells: Cells, pos: Position, rect: &Rect) {
    // TODO: Make it look like a tic tac toe board
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let rows = cells.iter().enumerate().map(|(i, item)| {
        // let height = item
        //     .iter()
        //     .map(|content| content.to_text(None).chars().filter(|c| *c == '\n').count())
        //     .max()
        //     .unwrap_or(0)
        //     + 1;
        let cells = item.iter().enumerate().map(|(j, c)| {
            Cell::from(Span::raw(c.to_text(Some((i, j))))).style(if (i, j) == pos.to_tuple() {
                selected_style
            } else {
                Style::default().fg(match c {
                    GameCell::Empty => Color::Gray,
                    GameCell::Cross => Color::Red,
                    GameCell::Circle => Color::Blue,
                })
            })
        });
        Row::new(cells).height(rect.height / 3)
    });
    let t = Table::new(rows)
        .block(Block::default().borders(Borders::ALL))
        .widths(&[Constraint::Ratio(1, 3); 3]);
    f.render_widget(t, *rect)
}

fn draw_info<B: Backend>(f: &mut Frame<B>, rect: &Rect, state: &GameState) {
    let info = match state {
        GameState::GameInProgress(_, _, _) => {
            "Game in progress...\nPress M/ Esc to open the Game Menu\nPress P to place a piece, Q to \
            quit, or R to reset the board.\nUse the arrow keys to move the piece."
                .to_string()
        }
        GameState::GameOver(..) => {
            "Game over!\nPress M/ Esc to open the Game Menu\nPress R to reset the board or Q to quit.".to_string()
        }
        // TODO:: Add Menu info
        // GameState::Menu(_) => "Tic Tac Toe Menu\nPress Q to quit, or use the up and down arrow keys to select an item."
        //     .to_string(),
    };
    let text_block = Paragraph::new(info).block(Block::default().title("Info").borders(Borders::ALL));
    f.render_widget(text_block, *rect);
}
