mod game;
use game::board::Board;

use std::fs::File;
use std::io;
use std::io::{BufWriter, Write};

/// Función que formatea el path de output del programa.
fn fmt_output(input: &str, output: &str) -> String {
    match input.split('/').nth_back(0) {
        Some(name) => String::from(output) + "/" + name,
        None => String::from(output),
    }
}

/// Procedimiento que escribe un string al path output.
fn write_to_file(output: &str, s: &str) -> io::Result<()> {
    let f = File::create(output)?;
    BufWriter::new(f).write_all(s.as_bytes())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 5 {
        eprintln!("Use: <input_file> <output_dir> X Y");
        return Ok(());
    }

    let input = &args[1];
    let output = fmt_output(input, &args[2]);

    let parse = |s: &String| s.parse::<i8>();
    let (x, y) = match (parse(&args[3]), parse(&args[4])) {
        (Ok(a), Ok(b)) => (a, b),
        _ => return write_to_file(&output, "ERROR: Coordinate is invalid"),
    };

    let mut board = match Board::new(input) {
        Err(e) => return write_to_file(&output, &e),
        Ok(board) => board,
    };

    board.pop((x, y));
    if let Err(e) = board.save(&output) {
        eprintln!("{e}");
    }

    Ok(())
}
