use bomberman::game::dir::Dir;

#[test]
fn move_pos() {
    let pos = (1, 1);
    assert_eq!(Dir::Up.move_pos(pos), (1, 0));
    assert_eq!(Dir::Down.move_pos(pos), (1, 2));
    assert_eq!(Dir::Left.move_pos(pos), (0, 1));
    assert_eq!(Dir::Right.move_pos(pos), (2, 1));
    assert_eq!(Dir::None.move_pos(pos), pos);
}
