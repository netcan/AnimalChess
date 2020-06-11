use std::time::Instant;
use crate::board::*;
use crate::player::*;
use crate::chess::{*, RoleType::*};
use rand::seq::SliceRandom;
use std::cell::RefCell;
use std::rc::Rc;

const MAX_DEPTH: i32 = 100;
const INF: ScoreType = 1000000;
const WIN_SCORE: ScoreType = INF - MAX_DEPTH;

type HisTblType = [[[ScoreType; COL_NUM]; ROW_NUM]; 16];

pub struct AlphaBeta {
    board: Rc<RefCell<Board>>,
    history_table: HisTblType,
    compture_mv: Option<MOVE>,
}

impl AlphaBeta {
    pub fn new(board: Rc<RefCell<Board>>) -> Self {
        Self {
            board,
            history_table: [[[0; COL_NUM]; ROW_NUM]; 16],
            compture_mv: None,
        }
    }

    fn generate_all_steps(&mut self) -> Vec<MOVE> {
        let mut moves = self.board.borrow().generate_all_steps();
        moves.sort_by(|&lhs, &rhs| {
            let lhs_his_score = *self.get_history_score(lhs);
            let rhs_his_score = *self.get_history_score(rhs);

            (rhs_his_score).cmp(&lhs_his_score)
        });
        moves
    }

    fn evaluate(&self) -> ScoreType {
        const CHESS_SCORE: [ScoreType; 8] = [
            // ELEPHANT, LION, TIGER, PANTHER, WOLF, DOG, CAT, RAT
            1000, 900, 800, 700, 600, 500, 400, 300
        ];
        // const POS_SCORE:
        const POS_SCORE: [[[ScoreType; COL_NUM]; ROW_NUM]; 8] = [
            // ELEPHANT
            [
                [ 0,  0,  0,  0,  0,  0,  0],
                [10, 10, 10, 10, 10, 10, 10],
                [20, 20, 20, 20, 20, 20, 20],
                [30, 30, 30, 30, 30, 30, 30],
                [40, 40, 40, 40, 40, 40, 40],
                [50, 50, 50, 50, 50, 50, 50],
                [60, 60, 60, 60, 60, 60, 60],
                [70, 70, 70, 70, 70, 70, 70],
                [80, 80, 80, 80, 80, 80, 80]
            ],
            // LION
            [
                [ 0,  0,  0,  0,  0,  0,  0],
                [10, 10, 10, 10, 10, 10, 10],
                [20, 20, 20, 20, 20, 20, 20],
                [30, 30, 30, 30, 30, 30, 30],
                [40, 40, 40, 40, 40, 40, 40],
                [50, 50, 50, 50, 50, 50, 50],
                [60, 60, 60, 60, 60, 60, 60],
                [70, 70, 70, 70, 70, 70, 70],
                [80, 80, 80, 80, 80, 80, 80]
            ],
            // TIGER
            [
                [ 0,  0,  0,  0,  0,  0,  0],
                [10, 10, 10, 10, 10, 10, 10],
                [20, 20, 20, 20, 20, 20, 20],
                [30, 30, 30, 30, 30, 30, 30],
                [40, 40, 40, 40, 40, 40, 40],
                [50, 50, 50, 50, 50, 50, 50],
                [60, 60, 60, 60, 60, 60, 60],
                [70, 70, 70, 70, 70, 70, 70],
                [80, 80, 80, 80, 80, 80, 80]
            ],
            // PANTHER
            [
                [ 15,  20,  15,  15,  15,  20,  15],
                [ 15,  15,  20,  15,  20,  15,  15],
                [ 15,  15,  15,  15,  15,  15,  15],
                [ 0,  0,  0,  0,  0,  0,  0],
                [ 0,  0,  0,  0,  0,  0,  0],
                [ 0,  0,  0,  0,  0,  0,  0],
                [ 0,  0,  0,  0,  0,  0,  0],
                [ 0,  0,  0,  0,  0,  0,  0],
                [ 0,  0,  0,  0,  0,  0,  0],
            ],
            // WOLF
            [
                [ 15,  20,  15,  15,  15,  20,  15],
                [ 15,  15,  20,  15,  20,  15,  15],
                [ 15,  15,  15,  15,  15,  15,  15],
                [ 0,  0,  0,  0,  0,  0,  0],
                [ 0,  0,  0,  0,  0,  0,  0],
                [ 0,  0,  0,  0,  0,  0,  0],
                [ 0,  0,  0,  0,  0,  0,  0],
                [ 0,  0,  0,  0,  0,  0,  0],
                [ 0,  0,  0,  0,  0,  0,  0],
            ],
            // DOG
            [
                [ 15,  20,  15,  15,  15,  20,  15],
                [ 15,  15,  20,  15,  20,  15,  15],
                [ 15,  15,  15,  15,  15,  15,  15],
                [ 0,  0,  0,  0,  0,  0,  0],
                [ 0,  0,  0,  0,  0,  0,  0],
                [ 0,  0,  0,  0,  0,  0,  0],
                [ 0,  0,  0,  0,  0,  0,  0],
                [ 0,  0,  0,  0,  0,  0,  0],
                [ 0,  0,  0,  0,  0,  0,  0],
            ],
            // CAT
            [
                [ 15,  20,  15,  15,  15,  20,  15],
                [ 15,  15,  20,  15,  20,  15,  15],
                [ 15,  15,  15,  15,  15,  15,  15],
                [ 0,  0,  0,  0,  0,  0,  0],
                [ 0,  0,  0,  0,  0,  0,  0],
                [ 0,  0,  0,  0,  0,  0,  0],
                [ 0,  0,  0,  0,  0,  0,  0],
                [ 0,  0,  0,  0,  0,  0,  0],
                [ 0,  0,  0,  0,  0,  0,  0],
           ],
            // RAT
            [
                [ 0,  0,  0,  0,  0,  0,  0],
                [10, 10, 10, 10, 10, 10, 10],
                [20, 20, 20, 20, 20, 20, 20],
                [30, 30, 30, 30, 30, 30, 30],
                [40, 40, 40, 40, 40, 40, 40],
                [50, 50, 50, 50, 50, 50, 50],
                [60, 60, 60, 60, 60, 60, 60],
                [70, 70, 70, 70, 70, 70, 70],
                [80, 80, 80, 80, 80, 80, 80]
            ],
        ];

        let mut score: ScoreType = 0;
        for i in 0..ROW_NUM {
            for j in 0..COL_NUM {
                let chess_id = self.board.borrow().chesses[i][j];
                if chess_id == EMPTY_CHESS { continue; }

                let chess_score = CHESS_SCORE[chess_id.kind.get_idx()];
                if chess_id.role == RED {
                    let pos_score = POS_SCORE[chess_id.kind.get_idx()][ROW_NUM - i - 1][j];
                    score += chess_score + pos_score;
                } else {
                    let pos_score = POS_SCORE[chess_id.kind.get_idx()][i][j];
                    score -= chess_score + pos_score;
                }
            }
        }

        if self.board.borrow().role == RED { score }
        else { -score }
    }

    fn get_history_score(&mut self, mv: MOVE) -> &mut ScoreType {
        let (src, dst) = get_move(mv);
        &mut self.history_table[
            self.board.borrow().chesses[src.0][src.1].get_chess_idx()
        ][dst.0][dst.1]
    }

    fn store_best_move(&mut self, mv: MOVE, depth: i32) {
        *self.get_history_score(mv) += depth * depth;
    }

    fn alpha_beta(&mut self,
        cur_depth: i32, depth: i32,
        mut alpha: ScoreType, beta: ScoreType) -> ScoreType {

        if cur_depth == depth { return self.evaluate(); }

        // 超出边界的alph-beta搜索
        let mut best_score = -INF;
        let mut best_move: Option<MOVE> = None;

        for mv in self.generate_all_steps() {
            self.board.borrow_mut().move_chess(mv);
            let score = -self.alpha_beta(cur_depth + 1, depth, -beta, -alpha);
            self.board.borrow_mut().undo_move();

            if score > best_score {
                best_score = score;
                if score >= beta {
                    best_move = Some(mv);
                    break;
                }
                if score > alpha {
                    best_move = Some(mv);
                    alpha = score;
                }
            }

        }
        if best_score == -INF { return cur_depth - INF; }

        if let Some(mv) = best_move {
            if cur_depth == 0 { self.compture_mv = Some(mv); }
            self.store_best_move(mv, depth - cur_depth);
        }
        best_score
    }

    pub fn search_main(&mut self) -> MOVE {
        // clean up
        self.history_table = [[[0; COL_NUM]; ROW_NUM]; 16];
        self.compture_mv = None;

        println!("search init board score = {}", self.evaluate());
        let timeout: i32 = 500 * 1000; // 200 ms
        let now = Instant::now();

        let mut max_depth = 0;
        let mut score = 0;
        for d in 1..=MAX_DEPTH {
            if now.elapsed().as_micros() as i32 >= timeout { break; }
            score = self.alpha_beta(0, d, -INF, INF);
            max_depth = d;
            if score >= WIN_SCORE || score <= -WIN_SCORE { break; }
        }

        println!("max_depth = {} find score = {}", max_depth, score);

        if let Some(mv) = self.compture_mv {
            mv
        } else {
            println!("not found solution, random move.");
            self.board.borrow()
                .generate_all_steps()
                .choose(&mut rand::thread_rng())
                .copied().expect("No Moveable!")
        }
    }
}


impl Player for AlphaBeta {
    fn get_move(&mut self) -> MOVE {
        self.search_main()
    }
}
