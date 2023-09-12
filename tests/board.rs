use bomberman::game::board::Board;
use std::fs::read_to_string;

#[test]
fn invalid_path() {
    assert!(Board::new("invalid_path.txt").is_err());
    assert!(Board::new("").is_err())
}

#[test]
fn invalid_board_shape() {
    assert!(Board::new("tests/boards/invalid.txt").is_err());
}

#[test]
fn read_from_file() {
    let board = Board::new("tests/boards/1.txt").unwrap();
    let s = read_to_string("tests/boards/1.txt").unwrap();
    assert_eq!(s, board.to_string());
}

#[test]
fn save() {
    let board = Board::new("tests/boards/savetest.txt").unwrap();
    board.save("tests/output/savetest.txt").unwrap();
    let s = read_to_string("tests/output/savetest.txt").unwrap();
    assert_eq!(board.to_string(), s);
}

#[test]
fn complete_example_1() {
    let mut board = Board::new("tests/boards/1.txt").unwrap();
    let s = read_to_string("tests/boards/1R.txt").unwrap();
    board.pop((0, 0));
    assert_eq!(s, board.to_string());
}

#[test]
fn complete_example_2() {
    let mut board = Board::new("tests/boards/2.txt").unwrap();
    let s = read_to_string("tests/boards/2R.txt").unwrap();
    board.pop((2, 4));
    assert_eq!(s, board.to_string());
}

#[test]
fn complete_example_3() {
    let mut board = Board::new("tests/boards/3.txt").unwrap();
    let s = read_to_string("tests/boards/3R.txt").unwrap();
    board.pop((0, 4));
    assert_eq!(s, board.to_string());
}

#[test]
fn complete_example_custom() {
    let mut board = Board::new("tests/boards/custom.txt").unwrap();
    let s = read_to_string("tests/boards/customR.txt").unwrap();
    board.pop((0, 0));
    assert_eq!(s, board.to_string());
}
