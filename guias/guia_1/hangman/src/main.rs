use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader, Result};

#[derive(Debug)]
struct Palabra {
    indices: Vec<bool>,
    texto: String,
    predichas: u8,
    vidas: u8,
    erradas: HashSet<char>,
}

fn leer_stdin() -> String {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Error al leer la entrada");
    input
}

fn print_juego(p: &Palabra) {
    let mut output = String::new();
    for (i, c) in p.texto.chars().enumerate() {
        if p.indices[i] {
            output.push(c);
        } else {
            output.push('_');
        }
    }

    print!("\x1B[2J\x1B[1;1H");
    println!("Vidas: {}", p.vidas);
    println!("Predichas: {}", p.predichas);
    println!("{}", output);
    println!("Erradas: {:?}", p.erradas);
    print!("\n> ");
    std::io::stdout().flush().unwrap();
}

fn hangman(mut p: Palabra) -> bool {
    while p.indices.contains(&false) {
        print_juego(&p);
        let input = leer_stdin();

        if input == "salir\n" {
            return true;
        }

        let mut erro = true;
        for (i, char_texto) in p.texto.chars().enumerate() {
            if input.len() > 2 {
                erro = false;
                break;
            }

            if char_texto == input.chars().next().unwrap() {
                if !p.indices[i] {
                    p.predichas += 1;
                    p.indices[i] = true;
                }

                erro = false;
            }
        }

        if erro {
            let letra = input.chars().nth(0).unwrap();
            if !p.erradas.contains(&letra) {
                p.vidas -= 1;
            }

            p.erradas.insert(letra);
        }

        if p.vidas == 0 {
            println!("Perdiste");
            return true;
        }
    }

    print_juego(&p);
    println!();
    false
}

fn main() -> Result<()> {
    let file = File::open("src/palabras.txt")?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let exit = match line {
            Ok(line) => hangman(Palabra {
                indices: vec![false; line.len()],
                erradas: HashSet::with_capacity(line.len()),
                texto: line,
                predichas: 0,
                vidas: 5,
            }),

            Err(e) => {
                println!("Error: {}", e);
                true
            }
        };

        if exit {
            break;
        }
    }

    Ok(())
}
