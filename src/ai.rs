use std::collections::LinkedList;
use crate::game::*;
use crate::chess::*;

type ScoreType = i32;
const INF: ScoreType = 1000000;

impl Game {
    fn generate_all_steps(&self) -> LinkedList<MOVE> {
        let mut moves = LinkedList::new();
        for i in 0..ROW_NUM {
            for j in 0..COL_NUM {
                let chess_id = self.chesses[i][j].id;
                if get_chess_role(chess_id) != self.role { continue }
                moves.extend(
                    self.generate_steps(&(i, j)).into_iter().map(|pos| {
                        ((i, j), pos)
                    })
                );
            }
        }

        moves
    }

    fn evaluate(&self) -> ScoreType {
        const CHESS_SCORE: [ScoreType; 8] = [
            100, 90, 80, 70, 60, 50, 40, 30
        ];

        let mut score: ScoreType = 0;
        for i in 0..ROW_NUM {
            for j in 0..COL_NUM {
                let chess_id = self.chesses[i][j].id;
                if chess_id == EMPTY { continue; }
                let chess_score = CHESS_SCORE[get_chess_type(chess_id) as usize];
                if get_chess_role(chess_id) == RED {
                    score += chess_score;
                } else {
                    score -= chess_score;
                }
            }
        }

        if self.role == RED { score }
        else { -score }
    }

    fn alpha_beta(&mut self, cur_depth: i32, depth: i32, mut alpha: ScoreType, beta: ScoreType) -> ScoreType {
        if cur_depth == depth { return self.evaluate(); }

        // 超出边界的alph-beta搜索
        let mut best_score = -INF;
        let mut best_move: Option<MOVE> = None;

        for mv in self.generate_all_steps() {
            self.move_chess(&mv);
            let score = -self.alpha_beta(cur_depth + 1, depth, -beta, -alpha);
            self.undo_move();

            if score > best_score {
                best_score = score;
                best_move = Some(mv);
                if score >= beta { break }
                if score > alpha { alpha = score; }
            }

        }

        if cur_depth == 0 {
            if let Some(mv) = best_move {
                self.move_chess(&mv)
            }
        }
        best_score
    }

    pub fn search_main(&mut self) {
        if self.compture_turn {
            self.compture_turn = false;
            self.alpha_beta(0, 1, -INF, INF);
        }
    }


}
