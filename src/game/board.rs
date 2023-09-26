use super::dir::Dir;
use super::obj::Obj;
use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display, Formatter},
    fs::{self, File},
    io::{BufWriter, Write},
    str::FromStr,
};

/// Tipo de dato del que es el tablero del
/// juego. K: Puntos (x, y), V: Tipo Obj.
type BoardMap = HashMap<(i32, i32), Obj>;

/// Struct que representa el tablero
/// del juego dentro del programa.
pub struct Board {
    board: BoardMap,
    n: usize,
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut output_str = String::new();
        let mut items = Vec::from_iter(self.board.iter());
        items.sort_by_key(|(pos, _)| pos.0 * self.n as i32 + pos.1);

        for (pos, obj) in items {
            let fmtobj = format!("{obj}");
            output_str.push_str(fmtobj.as_str());
            output_str.push(if pos.1 == self.n as i32 - 1 {
                '\n'
            } else {
                ' '
            });
        }

        write!(f, "{output_str}")
    }
}

impl FromStr for Board {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let n = s.lines().count();
        let mut board = BoardMap::new();
        // Itera las lineas del archivo separando por espacios
        // para obtener los objetos y sus coordenadas.
        for (i, line) in s.lines().enumerate() {
            let mut read = 0;
            for (j, obj) in line.split_whitespace().enumerate() {
                board.insert((i as i32, j as i32), Obj::from_str(obj)?);
                read += 1;
            }

            if n != read || n <= i {
                return Err("ERROR: Matrix is not square");
            }
        }

        Ok(Board { board, n })
    }
}

impl Board {
    /// Instancia un tablero desde un archivo.
    pub fn new(input: &str) -> Result<Board, &'static str> {
        match fs::read_to_string(input) {
            Err(_) => Err("ERROR: Input file not found"),
            Ok(content) => Self::from_str(&content),
        }
    }

    /// Guarda el tablero en path.
    pub fn save(&self, path: &str) -> Result<(), &'static str> {
        let f = File::create(path).map_err(|_| "ERROR: Creating file")?;
        BufWriter::new(f)
            .write_all(self.to_string().as_bytes())
            .map_err(|_| "Error: Writing to file")
    }

    /// Procedimiento que ejecuta la interacción entre una bomba y otro objeto.
    /// Devuelve Some(()) en caso de tener que seguir iterando sobre el tablero
    /// y None en caso de querer cortar la expansión.
    fn obj_interact(&mut self, bomb: &Obj, pos: (i32, i32), dir: &mut Dir) -> Option<()> {
        // Devolver None si la posición está vacia.
        let mut cell = self.board.get_mut(&pos)?;
        match (bomb, &mut cell) {
            (Obj::BreakBomb(_), Obj::Rock) => (),
            (_, Obj::BreakBomb(_) | Obj::Bomb(_)) => self.explode(pos),
            (_, Obj::Detour(new_dir)) => *dir = new_dir.clone(),
            (_, Obj::Empty) => (),
            (_, Obj::Enemy(hp)) => match hp {
                1 => *cell = Obj::Empty,
                _ => *hp -= 1,
            },
            (_, _) => return None,
        }

        Some(())
    }

    /// Procedimiento que propaga
    /// la explosión de una bomba
    /// del tablero.
    fn propagate(
        &mut self,
        steps: &mut HashSet<(i32, i32)>,
        pos: (i32, i32),
        bomb: &Obj,
        range: u32,
        mut dir: Dir,
    ) {
        // Se fija si esta explosión
        // ya pasó por esta celda.
        if !steps.contains(&pos) {
            match self.obj_interact(bomb, pos, &mut dir) {
                None => return,
                Some(_) => (),
            }

            steps.insert(pos);
        }

        // Si el rango es 0 entonces la
        // proxima llamada no es válida.
        if range == 0 {
            return;
        }

        // Llama a la función nuevamente con 1 menos de rango.
        self.propagate(steps, dir.move_pos(pos), bomb, range - 1, dir)
    }

    /// Procedimiento que activa una bomba
    /// del tablero, la consume y propaga
    /// su explosion hacia los 4 lados.
    fn explode(&mut self, pos: (i32, i32)) {
        if let Some(bomb) = self.board.remove(&pos) {
            let range = match bomb {
                Obj::Bomb(range) | Obj::BreakBomb(range) => range,
                _ => return,
            };

            self.board.insert(pos, Obj::Empty);
            let mut set = HashSet::with_capacity(2 * self.n);

            self.propagate(&mut set, pos, &bomb, range, Dir::Up);
            self.propagate(&mut set, pos, &bomb, range, Dir::Down);
            self.propagate(&mut set, pos, &bomb, range, Dir::Left);
            self.propagate(&mut set, pos, &bomb, range, Dir::Right);
        }
    }

    /// Procedimiento que verifica que la posición pasada sea válida
    /// e inicializa la explosión de la bomba.
    pub fn pop(&mut self, pos: (i32, i32)) -> Result<(), &'static str> {
        let pos = (pos.1, pos.0);
        // Verificar que en la posición dada
        // hay un Obj de tipo Bomb o BreakBomb.
        match self.board.get(&pos) {
            Some(Obj::Bomb(_) | Obj::BreakBomb(_)) => self.explode(pos),
            _ => return Err("Error: Invalid coordinates"),
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Board;
    use std::fs;

    #[test]
    fn to_string() {
        let board = Board::new("boards/1.txt").unwrap();
        let s = fs::read_to_string("boards/1.txt").unwrap();
        assert_eq!(board.to_string(), s);
    }

    #[test]
    fn invalid_path() {
        assert!(Board::new("invalid_path.txt").is_err());
        assert!(Board::new("").is_err());
    }

    #[test]
    fn invalid_board_shape() {
        assert!(Board::new("src/game/test_boards/invalid1.txt").is_err());
        assert!(Board::new("src/game/test_boards/invalid2.txt").is_err());
        assert!(Board::new("src/game/test_boards/invalid3.txt").is_err());
    }

    #[test]
    fn read_from_file() {
        let board = Board::new("boards/1.txt").unwrap();
        let s = fs::read_to_string("boards/1.txt").unwrap();
        assert_eq!(s, board.to_string());
    }

    #[test]
    fn save() {
        let board = Board::new("src/game/test_boards/savetest.txt").unwrap();
        let s = fs::read_to_string("output/savetest.txt").unwrap();
        board.save("output/savetest.txt").unwrap();
        assert_eq!(board.to_string(), s);
    }
}
