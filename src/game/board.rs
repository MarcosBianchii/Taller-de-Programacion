use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs::{read_to_string, File};
use std::io::{BufWriter, Write};

use super::obj::Obj;

type BoardMap = HashMap<(u8, u8), Obj>;

pub struct Board {
    pub board: BoardMap,
    output: String,
    n: u8,
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut output_str = String::new();
        let mut items = Vec::from_iter(self.board.iter());
        items.sort_by_key(|(pos, _)| pos.0 * self.n + pos.1);

        items.iter().for_each(|((_, j), obj)| {
            let fmtobj = obj.to_string();
            output_str.push_str(format!("{fmtobj} ").chars().as_str());
            if *j == self.n {
                output_str.push_str("\n");
            }
        });

        write!(f, "{}", output_str)
    }
}

impl Board {
    pub fn new(input: String, output: &String) -> Result<Board, String> {
        let file = match read_to_string(&input) {
            Err(_) => return Err(format!("ERROR: file not found <{input}>")),
            Ok(content) => content,
        };

        let mut game = Board {
            board: BoardMap::new(),
            output: output.clone(),
            n: 0,
        };

        for (i, line) in file.lines().enumerate() {
            for (j, obj) in line.split_whitespace().enumerate() {
                game.board.insert((i as u8, j as u8), Obj::from(obj));
            }

            game.n = u8::max(game.n, i as u8);
        }

        Ok(game)
    }

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
}
