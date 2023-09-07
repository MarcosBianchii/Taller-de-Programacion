use std::fmt;
use std::fmt::{Display, Formatter};

use super::dir::Dir;

/// Enum que representa un tipo de
/// objeto o "pieza" en el tablero.
pub enum Obj {
    Enemy(u8),
    Bomb(u8),
    BreakBomb(u8),
    Rock,
    Wall,
    Detour(Dir),
    Empty,
}

impl From<&str> for Obj {
    fn from(string: &str) -> Self {
        let x = string.chars().next();
        let y = &string[1..];

        let parse_u8 = |s: &str| match s.parse::<u8>() {
            Ok(num) => num,
            Err(_) => 0,
        };

        match (x, y) {
            (Some('F'), hp) => Obj::Enemy(parse_u8(hp)),
            (Some('B'), range) => Obj::Bomb(parse_u8(range)),
            (Some('S'), range) => Obj::BreakBomb(parse_u8(range)),
            (Some('D'), dir) => Obj::Detour(Dir::from(dir)),
            (Some('R'), _) => Obj::Rock,
            (Some('W'), _) => Obj::Wall,
            _ => Obj::Empty,
        }
    }
}

impl Display for Obj {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let output = match self {
            Obj::Enemy(hp) => format!("F{hp}"),
            Obj::Bomb(range) => format!("B{range}"),
            Obj::BreakBomb(range) => format!("S{range}"),
            Obj::Detour(dir) => format!("D{dir}"),
            Obj::Rock => "R".to_string(),
            Obj::Wall => "W".to_string(),
            Obj::Empty => "_".to_string(),
        };

        write!(f, "{output}")
    }
}
