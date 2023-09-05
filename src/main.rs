mod juego;
use juego::Juego;

fn main() {
    let juego = Juego::from_file("tableros/1.txt").unwrap();
    println!("{juego:?}");
}
