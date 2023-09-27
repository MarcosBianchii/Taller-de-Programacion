mod game;
use game::board::Board;
use std::{
    fs::File,
    io::{self, BufWriter, Write},
};

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

/// Función que parsea la posición de impacto.
fn parse_pos(x: &str, y: &str) -> Result<(i32, i32), &'static str> {
    let parse = |s: &str| {
        s.parse::<i32>()
            .map_err(|_| "ERROR: Invalid position arguments")
    };

    match (parse(x)?, parse(y)?) {
        (x @ 0.., y @ 0..) => Ok((x, y)),
        _ => Err("ERROR: Position should be positive for both arguments"),
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 5 {
        eprintln!("Use: <input_file> <output_dir> X Y");
        return Ok(());
    }

    let input = &args[1];
    let output = fmt_output(input, &args[2]);

    let (x, y) = match parse_pos(&args[3], &args[4]) {
        Err(e) => return write_to_file(&output, e),
        Ok((x, y)) => (x, y),
    };

    let mut board = match Board::new(input) {
        Err(e) => return write_to_file(&output, e),
        Ok(board) => board,
    };

    if let Err(e) = board.pop((x, y)) {
        return write_to_file(&output, e);
    }

    board.save(&output).unwrap_or_else(|e| eprintln!("{e}"));
    Ok(())
}
