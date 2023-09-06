use bomberman::game::board::Board;

#[test]
fn invalid_path() {
    assert!(Board::new("invalid_path.txt", "").is_err());
    assert!(Board::new("", "").is_err())
}

// #[test]
// fn invalid_output_path() {
//     let input = "tests/boards/1.txt";
//     let output = "invalid_output";
//     let board = Board::new(input, output).unwrap();
//     assert!(board.save().is_err());
// }
