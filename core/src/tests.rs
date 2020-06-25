/*************************************************************************
    > File Name: tests.rs
    > Author: Netcan
    > Descripton: Board Tests
    > Blog: http://www.netcan666.com
    > Mail: 1469709759@qq.com
    > Created Time: 2020-06-20 19:22
************************************************************************/

#[test]
fn test_encode_decode_move() {
    use crate::board::{Board, get_move};
    use rand::seq::SliceRandom;
    for _ in 0..50 {
        let mut board = Board::new();
        loop {
            let steps = board.generate_all_steps();
            if steps.is_empty() { break; }
            for &mv in &steps {
                let encode_mv = board.encode_move(mv);
                let decode_mv = board.decode_move(encode_mv);
                assert!(encode_mv <= 252, "encode_mv should <= 252!");
                assert_eq!(decode_mv, mv,
                    "{:?} != {:?}", get_move(decode_mv), get_move(mv));
            }
            board.move_chess(*steps.choose(&mut rand::thread_rng()).unwrap());
        }
    }
}

#[test]
fn test_load_and_get_fen() {
    use crate::board::{Board, ROW_NUM, COL_NUM};
    use rand::seq::SliceRandom;
    for _ in 0..50 {
        let mut board = Board::new();
        loop {
            let steps = board.generate_all_steps();
            if steps.is_empty() { break; }
            for &mv in &steps {
                board.move_chess(mv);
                let fen = board.get_fen();
                let mut board_expected = Board::new();
                board_expected.load_fen(&fen);
                assert_eq!(fen, board_expected.get_fen());
                assert_eq!(board.role, board_expected.role);

                assert_eq!(board.chesses, board_expected.chesses);
                board.undo_move();
            }
            board.move_chess(*steps.choose(&mut rand::thread_rng()).unwrap());
        }
    }

}
