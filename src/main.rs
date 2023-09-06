mod game;
use game::board::Board;

use std::fs::File;
use std::io;
use std::io::{BufWriter, Write};

/// Funcion que formatea el path de output del programa.
fn fmt_output(input: &String, output: &String) -> String {
    match input.split('/').nth_back(0) {
        Some(name) => String::from(output) + "/" + name,
        None => "".to_string(), // Unreachable.
    }
}

/// Procedimiento que escribe un String al path output.
fn write_to_file(output: &String, s: String) -> io::Result<()> {
    let f = File::create(output)?;
    BufWriter::new(f).write_all(s.as_bytes())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Uso: <input_file> <output_dir> X Y");
        return Ok(());
    }

    let input = &args[0];
    let output = fmt_output(&input, &args[1]);

    let parse = |s: &String| s.parse::<i8>();
    let (x, y) = match (parse(&args[2]), parse(&args[3])) {
        (Ok(a), Ok(b)) => (a, b),
        _ => (-1, -1),
    };

    let mut board = match Board::new(&input, &output) {
        Err(e) => return write_to_file(&output, e),
        Ok(board) => board,
    };

    board.execute((x, y));

    match board.save() {
        Err(e) => return write_to_file(&output, e),
        Ok(_) => Ok(()),
    }
}
