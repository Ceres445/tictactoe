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

pub enum Move {
    Down,
    Up,
    Left,
    Right,
    Place,
    PlaceAt(Position),
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

pub enum Action {
    Move(Move),
    ToggleMenu,
    Select(u8),
    Quit,
    Reset,
}
