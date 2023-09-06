use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs::{read_to_string, File};
use std::io::{BufWriter, Write};

use super::dir::Dir;
use super::obj::Obj;

/// Tipo de dato que se usa como key del
/// HashMap del tablero. Estos puntos
/// tienen que ser signed int porque el
/// HashMap necesita poder hacer get a
/// puntos negativos.
type Point = (i8, i8);

/// Tipo de dato del que es el tablero del
/// juego. K: Puntos (x, y), V: Tipo Obj.
type BoardMap = HashMap<Point, Obj>;

/// Struct que representa el tablero
/// del juego dentro del programa.
pub struct Board {
    board: BoardMap,
    output: String,
    n: u8,
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut output_str = String::new();
        let mut items = Vec::from_iter(self.board.iter());
        items.sort_by_key(|(pos, _)| pos.0 + pos.1 * self.n as i8);

        items.iter().for_each(|((j, _), obj)| {
            let fmtobj = obj.to_string();
            output_str.push_str(format!("{fmtobj} ").chars().as_str());
            if *j == self.n as i8 - 1 {
                output_str.push_str("\n");
            }
        });

        write!(f, "{}", output_str)
    }
}

impl Board {
    /// Instancia un tablero desde un archivo y su ruta para ser guardado.
    /// Devuelve Err en caso de no encontrar el archivo.
    pub fn new(input: &String, output: &String) -> Result<Board, String> {
        let file = match read_to_string(input) {
            Err(_) => return Err(format!("ERROR: file not found <{input}>")),
            Ok(content) => content,
        };

        let mut game = Board {
            board: BoardMap::new(),
            output: output.clone(),
            n: 0,
        };

        // Itera las lineas del archivo separando por espacios
        // para separar los objetos del tablero y guardar sus
        // coordenadas y tipos en el HashMap del tablero.
        for (i, line) in file.lines().enumerate() {
            for (j, obj) in line.split_whitespace().enumerate() {
                game.board.insert((j as i8, i as i8), Obj::from(obj));
            }

            // Guarda el n para poder formatear el tablero al guardarlo.
            game.n = i8::max(1 + game.n as i8, i as i8) as u8;
        }

        Ok(game)
    }

    /// Guarda el tablero en su path de output.
    /// Devuelve Err en caso de ser invalido el
    /// directorio de output.
    pub fn save(&self) -> Result<(), String> {
        let f = match File::create(&self.output) {
            Err(_) => return Err("ERROR: Output is invalid".to_string()),
            Ok(file) => file,
        };

        match BufWriter::new(f).write_all(self.to_string().as_bytes()) {
            Err(_) => return Err("ERROR: Output is invalid".to_string()),
            Ok(_) => Ok(()),
        }
    }

    /// Devuelve true si la coordenada recibida esta fuera
    /// de rango para la "matriz" de coordenadas del tablero.
    fn out_of_bounds(&self, pos: &Point) -> bool {
        !(pos.0 >= 0 && pos.0 <= self.n as i8 && pos.1 >= 0 && pos.1 <= self.n as i8)
    }

    /// Procedimiento recursivo que viaja por el HashMap interactuando
    /// con los objetos del tablero.
    fn execute_cell(
        &mut self,
        steps: &mut HashSet<Point>,
        pos: Point,
        bomb: &Obj,
        range: i8,
        mut dir: Dir,
    ) {
        // Si estamos fuera de rango o se
        // termino el rango de la bomba retorna.
        if self.out_of_bounds(&pos) || range < 0 {
            return;
        }

        // Se fija si esta explosion
        // ya paso por esta celda.
        if !steps.contains(&pos) {
            steps.insert(pos);

            // Interactua con el objeto en esta celda dependiendo
            // de si la explosion de la bomba fue una bomba normal
            // o una bomba de traspaso.
            if let Some(cur_spot) = self.board.get_mut(&pos) {
                match (bomb, cur_spot) {
                    (_, Obj::Bomb(_)) => self.execute(pos),
                    (_, Obj::BreakBomb(_)) => self.execute(pos),
                    (_, Obj::Detour(new_dir)) => dir = new_dir.clone(),
                    (_, Obj::Wall) => return,
                    (_, Obj::Enemy(hp)) => {
                        *hp -= 1;
                        if *hp == 0 {
                            self.board.insert(pos, Obj::Empty);
                        }
                    }
                    (Obj::BreakBomb(_), Obj::Rock) => (),
                    (Obj::Bomb(_), Obj::Rock) => return,
                    (_, _) => (),
                }
            }
        }

        // LLama a la funcion de nuevo pero con 1 menos de rango.
        self.execute_cell(steps, dir.move_pos(pos), bomb, range - 1, dir);
    }

    /// Procedimiento que comienza con la explosion de
    /// una de las bombas en el tablero, consume la bomba
    /// y expande su explosion hacia los 4 lados.
    pub fn execute(&mut self, pos: Point) {
        let mut set = HashSet::new();
        if let Some(bomb) = self.board.remove(&pos) {
            let range = match bomb {
                Obj::Bomb(range) => range as i8,
                Obj::BreakBomb(range) => range as i8,
                _ => 0,
            };

            self.board.insert(pos, Obj::Empty);
            self.execute_cell(&mut set, pos, &bomb, range, Dir::Up);
            self.execute_cell(&mut set, pos, &bomb, range, Dir::Down);
            self.execute_cell(&mut set, pos, &bomb, range, Dir::Left);
            self.execute_cell(&mut set, pos, &bomb, range, Dir::Right);
        }
    }
}
