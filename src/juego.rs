use std::collections::HashMap;
use std::fs::read_to_string;

#[derive(Debug)]
pub enum Objeto {
    Enemigo(u8),
    Bomba(u8),
    BombaTraspaso(u8),
    Roca,
    Pared,
    Desvio(char),
    Vacio,
}

pub type Tablero = HashMap<(u8, u8), Objeto>;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Juego {
    tablero: Tablero,
}

impl Juego {
    pub fn from_file(path: &str) -> Option<Juego> {
        let string = match read_to_string(path) {
            Ok(string) => string,
            Err(_) => return None,
        };

        let mut tablero = Tablero::new();
        for (i, linea) in string.lines().enumerate() {
            for (j, objeto) in linea.split_whitespace().enumerate() {
                let x = objeto.chars().nth(0);
                let y = objeto.chars().nth(1);

                let objeto = match (x, y) {
                    (Some('F'), Some(hp)) => Objeto::Enemigo(hp.to_digit(10)? as u8),
                    (Some('B'), Some(rango)) => Objeto::Bomba(rango.to_digit(10)? as u8),
                    (Some('S'), Some(rango)) => Objeto::BombaTraspaso(rango.to_digit(10)? as u8),
                    (Some('D'), Some(dir)) => Objeto::Desvio(dir),
                    (Some('R'), _) => Objeto::Roca,
                    (Some('W'), _) => Objeto::Pared,
                    _ => Objeto::Vacio,
                };

                tablero.insert(((i + 1) as u8, (j + 1) as u8), objeto);
            }
        }

        Some(Juego { tablero })
    }

    #[allow(dead_code)]
    pub fn at(&self, index: (u8, u8)) -> Option<&Objeto> {
        self.tablero.get(&(index.0, index.1))
    }
}
