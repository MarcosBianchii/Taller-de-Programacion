use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

/// Enum que representa un
/// sentido de dirección.
#[derive(Clone)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Display for Dir {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let c = match self {
            Dir::Up => "U",
            Dir::Down => "D",
            Dir::Left => "L",
            Dir::Right => "R",
        };

        write!(f, "{c}")
    }
}

impl FromStr for Dir {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().chars().next() {
            Some('U') => Ok(Dir::Up),
            Some('D') => Ok(Dir::Down),
            Some('L') => Ok(Dir::Left),
            Some('R') => Ok(Dir::Right),
            _ => Err("ERROR: Invalid Direction char"),
        }
    }
}

impl Dir {
    /// Función que devuelve un punto
    /// movido en dirección self.
    pub fn move_pos(&self, pos: (i32, i32)) -> (i32, i32) {
        match self {
            Dir::Up => (pos.0 - 1, pos.1),
            Dir::Down => (pos.0 + 1, pos.1),
            Dir::Left => (pos.0, pos.1 - 1),
            Dir::Right => (pos.0, pos.1 + 1),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Dir;

    #[test]
    fn move_pos() {
        let pos = (1, 1);
        assert_eq!(Dir::Up.move_pos(pos), (0, 1));
        assert_eq!(Dir::Down.move_pos(pos), (2, 1));
        assert_eq!(Dir::Left.move_pos(pos), (1, 0));
        assert_eq!(Dir::Right.move_pos(pos), (1, 2));

        let pos = (0, 0);
        assert_eq!(Dir::Up.move_pos(pos), (-1, 0));
        assert_eq!(Dir::Down.move_pos(pos), (1, 0));
        assert_eq!(Dir::Left.move_pos(pos), (0, -1));
        assert_eq!(Dir::Right.move_pos(pos), (0, 1));
    }
}
