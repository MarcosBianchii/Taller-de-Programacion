use std::fmt;
use std::fmt::{Display, Formatter};

/// Enum que representa una direccion
/// para los Obj::Detour.
#[derive(Clone, Copy)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
    None,
}

impl From<char> for Dir {
    fn from(c: char) -> Self {
        match c.to_uppercase().nth(0) {
            Some('U') => Dir::Up,
            Some('D') => Dir::Down,
            Some('L') => Dir::Left,
            Some('R') => Dir::Right,
            _ => Dir::None,
        }
    }
}

impl Display for Dir {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let c = match self {
            Dir::Up => "U",
            Dir::Down => "D",
            Dir::Left => "L",
            Dir::Right => "R",
            Dir::None => "_",
        };

        write!(f, "{c}")
    }
}

impl Dir {
    /// Funcion que devuelve un punto habiendo
    /// sido movido en direccion self.
    pub fn move_pos(&self, pos: (i8, i8)) -> (i8, i8) {
        match self {
            Dir::Up => (pos.0, pos.1 - 1),
            Dir::Down => (pos.0, pos.1 + 1),
            Dir::Left => (pos.0 - 1, pos.1),
            Dir::Right => (pos.0 + 1, pos.1),
            Dir::None => (-1, -1),
        }
    }
}
