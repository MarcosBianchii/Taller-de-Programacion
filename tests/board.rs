use bomberman::game::board::Board;
use std::fs;

#[test]
fn complete_example_1() {
    let mut board = Board::new("boards/1.txt").unwrap();
    let s = fs::read_to_string("tests/boards/1R.txt").unwrap();
    board.pop((0, 0)).unwrap();
    assert_eq!(s, board.to_string());
}

#[test]
fn complete_example_2() {
    let mut board = Board::new("boards/2.txt").unwrap();
    let s = fs::read_to_string("tests/boards/2R.txt").unwrap();
    board.pop((2, 4)).unwrap();
    assert_eq!(s, board.to_string());
}

#[test]
fn complete_example_3() {
    let mut board = Board::new("boards/3.txt").unwrap();
    let s = fs::read_to_string("tests/boards/3R.txt").unwrap();
    board.pop((0, 4)).unwrap();
    assert_eq!(s, board.to_string());
}

#[test]
fn complete_example_custom() {
    let mut board = Board::new("tests/boards/custom.txt").unwrap();
    let s = fs::read_to_string("tests/boards/customR.txt").unwrap();
    board.pop((0, 0)).unwrap();
    assert_eq!(s, board.to_string());
}
