use super::dir::Dir;
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

/// Enum que representa un tipo de
/// objeto o "pieza" en el tablero.
pub enum Obj {
    Enemy(u8),
    Bomb(u32),
    BreakBomb(u32),
    Rock,
    Wall,
    Detour(Dir),
    Empty,
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

impl FromStr for Obj {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_uppercase();
        Ok(match (&s[..1], &s[1..]) {
            ("F", hp) => Obj::Enemy(Self::parse_hp(hp)?),
            ("B", range) => Obj::Bomb(Self::parse_range(range)?),
            ("S", range) => Obj::BreakBomb(Self::parse_range(range)?),
            ("D", direc) => Obj::Detour(Dir::from_str(direc)?),
            ("R", "") => Obj::Rock,
            ("W", "") => Obj::Wall,
            ("_", "") => Obj::Empty,
            (_, _) => return Err("ERROR: Invalid entry in matrix"),
        })
    }
}

impl Obj {
    fn parse_hp(hp: &str) -> Result<u8, &'static str> {
        let hp: u8 = hp
            .parse()
            .map_err(|_| "ERROR: Invalid entry in Enemy object")?;

        match hp {
            1..=3 => Ok(hp),
            _ => Err("ERROR: Invalid hp amount for enemy"),
        }
    }

    fn parse_range(range: &str) -> Result<u32, &'static str> {
        let range: u32 = range
            .parse()
            .map_err(|_| "ERROR: Invalid entry in Bomb object")?;

        match range {
            1.. => Ok(range),
            0 => Err("ERROR: Invalid range for bomb"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Obj;
    use std::str::FromStr;

    #[test]
    fn from_str() {
        assert!(Obj::from_str("FE").is_err());
        assert!(Obj::from_str("F0").is_err());
        assert!(Obj::from_str("B0").is_err());
        assert!(Obj::from_str("B100").is_ok());
        assert!(Obj::from_str("F1").is_ok());
        assert!(Obj::from_str("_1").is_err());
        assert!(Obj::from_str("F-1").is_err());
        assert!(Obj::from_str("R ").is_err());
    }

    #[test]
    fn parse_hp() {
        assert!(Obj::parse_hp("-1").is_err());
        assert!(Obj::parse_hp("0").is_err());
        assert!(Obj::parse_hp("1").is_ok());
        assert!(Obj::parse_hp("2").is_ok());
        assert!(Obj::parse_hp("3").is_ok());
        assert!(Obj::parse_hp("4").is_err());
    }

    #[test]
    fn parse_range() {
        assert!(Obj::parse_range("-1").is_err());
        assert!(Obj::parse_range("0").is_err());
        assert!(Obj::parse_range("1").is_ok());
        assert!(Obj::parse_range("99999999").is_ok());
    }
}
