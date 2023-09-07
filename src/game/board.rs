use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs::{read_to_string, File};
use std::io::{BufWriter, Write};

use super::dir::Dir;
use super::obj::Obj;

/// Tipo de dato del que es el tablero del
/// juego. K: Puntos (x, y), V: Tipo Obj.
type BoardMap = HashMap<(i8, i8), Obj>;

/// Struct que representa el tablero
/// del juego dentro del programa.
pub struct Board {
    board: BoardMap,
    n: u8,
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut output_str = String::new();
        let mut items = Vec::from_iter(self.board.iter());
        items.sort_by_key(|(pos, _)| pos.0 + pos.1 * self.n as i8);

        for (pos, obj) in items {
            let fmtobj = format!("{obj}");
            output_str.push_str(fmtobj.as_str());
            output_str.push(if pos.0 == self.n as i8 - 1 { '\n' } else { ' ' });
        }

        write!(f, "{output_str}")
    }
}

impl Board {
    /// Valida que n es entero.
    fn validate_n(n: f32) -> Result<u8, String> {
        if n != n.round() {
            return Err("ERROR: Board shape is not squared".to_string());
        }

        Ok(n as u8)
    }

    /// Instancia un tablero desde un archivo.
    pub fn new(input: &str) -> Result<Board, String> {
        let file = match read_to_string(input) {
            Err(_) => return Err(format!("ERROR: file not found <{input}>")),
            Ok(content) => content,
        };

        let mut board = BoardMap::new();
        // Itera las lineas del archivo separando por espacios
        // para obtener los objetos y sus coordenadas.
        for (i, line) in file.lines().enumerate() {
            for (j, obj) in line.split_whitespace().enumerate() {
                board.insert((j as i8, i as i8), Obj::from(obj));
            }
        }

        // Valida que n es potencia de un entero.
        let n = Self::validate_n((board.len() as f32).sqrt())?;
        Ok(Board { board, n })
    }

    /// Guarda el tablero en path.
    pub fn save(&self, path: &str) -> Result<(), String> {
        let f = match File::create(path) {
            Err(_) => return Err("ERROR: Output is invalid".to_string()),
            Ok(file) => file,
        };

        match BufWriter::new(f).write_all(self.to_string().as_bytes()) {
            Err(_) => Err("ERROR: Output is invalid".to_string()),
            Ok(_) => Ok(()),
        }
    }

    /// Devuelve true si la coordenada recibida está fuera de rango.
    fn out_of_bounds(&self, pos: &(i8, i8)) -> bool {
        !(pos.0 >= 0 && pos.0 <= self.n as i8 && pos.1 >= 0 && pos.1 <= self.n as i8)
    }

    /// Procedimiento que ejecuta la interacción entre una bomba y otro objeto.
    fn obj_interact(&mut self, bomb: &Obj, pos: (i8, i8), dir: &mut Dir) -> Option<()> {
        if let Some(mut cur_cell) = self.board.get_mut(&pos) {
            match (bomb, &mut cur_cell) {
                (Obj::BreakBomb(_), Obj::Rock) => (),
                (Obj::Bomb(_), Obj::Rock) => return None,
                (_, Obj::BreakBomb(_) | Obj::Bomb(_)) => self.pop(pos),
                (_, Obj::Detour(new_dir)) => *dir = new_dir.clone(),
                (_, Obj::Wall) => return None,
                (_, Obj::Enemy(hp)) => match hp {
                    1 => *cur_cell = Obj::Empty,
                    _ => *hp -= 1,
                },
                (_, _) => (),
            }
        }

        Some(())
    }

    /// Procedimiento que propaga la explosión
    /// de una bomba del tablero.
    fn propagate(
        &mut self,
        steps: &mut HashSet<(i8, i8)>,
        pos: (i8, i8),
        bomb: &Obj,
        range: i8,
        mut dir: Dir,
    ) {
        // Si estamos fuera de rango del tablero o
        // se terminó el rango de la bomba retorna.
        if self.out_of_bounds(&pos) || range < 0 {
            return;
        }

        // Se fija si esta explosión
        // ya pasó por esta celda.
        if !steps.contains(&pos) {
            steps.insert(pos);
            match self.obj_interact(bomb, pos, &mut dir) {
                None => return,
                Some(_) => (),
            }
        }

        // Llama a la función nuevamente con 1 menos de rango.
        self.propagate(steps, dir.move_pos(pos), bomb, range - 1, dir);
    }

    /// Procedimiento que activa una bomba del tablero, la
    /// consume y propaga su explosion hacia los 4 lados.
    pub fn pop(&mut self, pos: (i8, i8)) {
        let mut set = HashSet::with_capacity(2 * self.n as usize);
        if let Some(bomb) = self.board.remove(&pos) {
            let range = match bomb {
                Obj::Bomb(range) => range as i8,
                Obj::BreakBomb(range) => range as i8,
                _ => 0,
            };

            self.board.insert(pos, Obj::Empty);
            self.propagate(&mut set, pos, &bomb, range, Dir::Up);
            self.propagate(&mut set, pos, &bomb, range, Dir::Down);
            self.propagate(&mut set, pos, &bomb, range, Dir::Left);
            self.propagate(&mut set, pos, &bomb, range, Dir::Right);
        }
    }
}
