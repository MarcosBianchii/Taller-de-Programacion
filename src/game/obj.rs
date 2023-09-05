use std::fmt;
use std::fmt::{Display, Formatter};

pub enum Obj {
    Enemy(u8),
    Bomb(u8),
    BreakBomb(u8),
    Rock,
    Wall,
    Detour(char),
    Empty,
}

impl From<&str> for Obj {
    fn from(string: &str) -> Self {
        let to_u8 = |c: char| c as u8 - '0' as u8;
        let x = string.chars().nth(0);
        let y = string.chars().nth(1);

        match (x, y) {
            (Some('F'), Some(hp)) => Obj::Enemy(to_u8(hp)),
            (Some('B'), Some(range)) => Obj::Bomb(to_u8(range)),
            (Some('S'), Some(range)) => Obj::BreakBomb(to_u8(range)),
            (Some('D'), Some(dir)) => Obj::Detour(dir),
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
