mod game;
#[allow(unused_imports)]
use game::board::Board;

use std::fs::File;
use std::io;
use std::io::{BufWriter, Write};

fn fmt_output(input: &String, output: &String) -> String {
    match input.split('/').nth_back(0) {
        Some(name) => name.to_string() + output,
        None => "".to_string(), // Unreachable.
    }
}

fn write_to_file(output: &String, s: String) -> io::Result<()> {
    let f = File::create(output)?;
    BufWriter::new(f).write_all(s.as_bytes())
}

#[allow(unused_variables)]
fn main() -> io::Result<()> {
    // let args: Vec<String> = std::env::args().collect();
    // if args.len() < 4 {
    //     eprintln!("Uso: <input_file> <output_dir> X Y");
    //     return;
    // }

    // let input = &args[0];
    // let output = fmt_output(&input, &args[1]);
    // let x = u8::from_str_radix(&args[2], 10);
    // let y = u8::from_str_radix(&args[3], 10);

    let input = "boards/1.txt".to_string(); // &args[0];
    let output = fmt_output(&input, &"output".to_string()); // &args[1];
    let x = 0;
    let y = 0;

    let board = match Board::new(input, &output) {
        Err(e) => return write_to_file(&output, e),
        Ok(board) => board,
    };

    // board.execute(x, y);

    match board.save() {
        Err(e) => return write_to_file(&output, e),
        Ok(_) => Ok(()),
    }
}
