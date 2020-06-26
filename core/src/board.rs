/*************************************************************************
    > File Name: board.rs
    > Author: Netcan
    > Descripton: Board impl
    > Blog: http://www.netcan666.com
    > Mail: 1469709759@qq.com
    > Created Time: 2020-06-20 19:23
************************************************************************/

use crate::chess::{*, ChessKind::*, RoleType::*};
use std::cmp::Ordering;
use std::collections::HashMap;
use rand::Rng;

pub type POS = u8;
pub type MOVE = u16;
pub type ScoreType = i32;

pub const ROW_NUM: usize = 9;
pub const COL_NUM: usize = 7;

pub const RED_DEN:   POS = 0x83;
pub const BLACK_DEN: POS = 0x3;
pub const TRAP: u64 = 0x1410000000000414;

pub fn get_pos(pos: POS) -> (usize, usize) {
    ((pos >> 4) as usize, (pos & 0xf) as usize)
}

pub fn get_move(mv: MOVE) -> ((usize, usize), (usize, usize)) {
    let (src, dst) = (get_src_pos(mv), get_dst_pos(mv));
    (get_pos(src as POS), get_pos(dst as POS))
}

pub fn get_src_pos(mv: MOVE) -> POS {
    (mv >> 8) as POS
}

pub fn get_dst_pos(mv: MOVE) -> POS {
    (mv & 0xff) as POS
}

pub fn to_pos(pos: &(usize, usize)) -> POS {
    ((pos.0 << 4) | pos.1) as POS
}

pub fn to_move(mv: &((usize, usize), (usize, usize))) -> MOVE {
    ((to_pos(&mv.0) as MOVE) << 8) | to_pos(&mv.1) as MOVE
}

type ZobristKeyType = u64;

#[derive(Clone)]
struct Context {
    eated: ChessId,
    zobrist_key: ZobristKeyType,
    mv: MOVE,
}

impl Context {
    fn new(eated: ChessId, zobrist_key: ZobristKeyType, mv: MOVE) -> Self {
        Self { eated, zobrist_key, mv }
    }
}

#[derive(Clone)]
pub struct Board {
    pub chesses: [[ChessId; COL_NUM]; ROW_NUM],
    pub role: RoleType, // 轮到谁下
    red_chess_num: usize,
    black_chess_num: usize,
    zobrist_tbl: [[[ZobristKeyType; COL_NUM]; ROW_NUM]; 16],
    pub zobrist_key: ZobristKeyType,
    in_den: RoleType,
    dup_counter: HashMap<ZobristKeyType, u8>,
    ctx: Vec<Context>,
}

enum UpdateChess {
    ADD,
    DEC
}

impl Board {
    // down, right, up, left
    const DXY: [(i8, i8); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    fn update_chess_num(&mut self, chess_id: ChessId, u: UpdateChess) {
        let chess_num = match chess_id.role {
            RED => &mut self.red_chess_num,
            BLACK => &mut self.black_chess_num,
            _ => return
        };

        use UpdateChess::*;
        match u {
            ADD => *chess_num += 1,
            DEC => *chess_num -= 1,
        }
    }

    // l5t/1d3c1/r1p1w1e/7/7/7/E1W1P1R/1C3D1/T5L w
    pub fn load_fen(&mut self, fen: &str) {
        self.chesses = [[EMPTY_CHESS; COL_NUM]; ROW_NUM];
        self.zobrist_key = 0;
        let fen_u8 = fen.as_bytes();
        let mut fen_idx = 0;

        let get_role = |c: u8| -> RoleType {
            if (c as char).is_lowercase() { BLACK }
            else { RED }
        };

        let mut pos = 0usize;
        while fen_idx < fen_u8.len() {
            let mut chess_id = EMPTY_CHESS;
            match fen_u8[fen_idx] {
                c @ b'e' | c @ b'E' => { chess_id = ChessId { role: get_role(c), kind: ELEPHANT }; }
                c @ b'l' | c @ b'L' => { chess_id = ChessId { role: get_role(c), kind: LION     }; }
                c @ b't' | c @ b'T' => { chess_id = ChessId { role: get_role(c), kind: TIGER    }; }
                c @ b'p' | c @ b'P' => { chess_id = ChessId { role: get_role(c), kind: PANTHER  }; }
                c @ b'w' | c @ b'W' => { chess_id = ChessId { role: get_role(c), kind: WOLF     }; }
                c @ b'd' | c @ b'D' => { chess_id = ChessId { role: get_role(c), kind: DOG      }; }
                c @ b'c' | c @ b'C' => { chess_id = ChessId { role: get_role(c), kind: CAT      }; }
                c @ b'r' | c @ b'R' => { chess_id = ChessId { role: get_role(c), kind: RAT      }; }
                n @ b'1' ..= b'9'   => { pos += (n - b'0') as usize; }
                b'/' => { }
                b' ' => { break; }
                _    => { unreachable!() }
            }

            if chess_id != EMPTY_CHESS {
                self.update_chess_num(chess_id, UpdateChess::ADD);
                self.chesses[pos / COL_NUM][pos % COL_NUM] = chess_id;
                self.zobrist_key ^= self.zobrist_tbl[chess_id.get_chess_idx()][pos / COL_NUM][pos % COL_NUM];
                pos += 1;
            }
            fen_idx += 1;
        }
        fen_idx += 1; // eat ' '
        self.role = if fen_u8[fen_idx] == b'w' { RED }
                    else { BLACK };

        // TODO: in_den check
        self.ctx.clear();
    }

    pub fn get_fen(&self) -> String {
	let mut ret = String::new();
	for i in 0..ROW_NUM {
	    let mut count = 0;
	    for j in 0..COL_NUM {
		let chess_id = self.chesses[i][j];
		if chess_id == EMPTY_CHESS {
		    count += 1;
		    continue;
		}

		if count > 0 { ret += &count.to_string(); }
		count = 0;
		let c = match chess_id.kind {
		    ELEPHANT => 'E',
		    LION => 'L',
		    TIGER => 'T',
		    PANTHER => 'P',
		    WOLF => 'W',
		    DOG => 'D',
		    CAT => 'C',
		    RAT => 'R',
		    _ => unreachable!(),
		};
		let c = if chess_id.role == BLACK {
		    c.to_ascii_lowercase()
		} else { c };
		ret += &c.to_string();
	    }
	    if count > 0 { ret += &count.to_string(); }
	    if i + 1 != ROW_NUM { ret += "/"; }
	}
	ret += &format!(" {}", if self.role == RED { 'w' } else { 'b' });
	ret
    }


    pub fn get_dup_count(&self) -> u8 {
        return *self.dup_counter.get(&self.zobrist_key).unwrap_or(&0);
    }

    pub fn get_step_count(&self) -> u8 {
        self.ctx.len() as u8
    }

    pub fn check_win(&self) -> RoleType {
        if self.in_den != RoleType::EMPTY { return self.in_den; }

        if self.red_chess_num * self.black_chess_num == 0 {
            if self.red_chess_num > 0 { return RED; }
            else { return BLACK; }
        }

        // if duplicate 2 times, first role loss
        let dup_times = self.dup_counter.get(&self.zobrist_key).unwrap_or(&0);
        if dup_times >= &2 {
            if self.role == RED { // red loss
                return BLACK;
            } else {
                return RED;
            }
        }

        RoleType::EMPTY
    }

    fn gen_zobrist_tbl() -> [[[ZobristKeyType; COL_NUM]; ROW_NUM]; 16] {
        let mut zobrist_tbl = [[[0; COL_NUM]; ROW_NUM]; 16];
        let mut rng = rand::thread_rng();

        for k in 0..16 {
            for i in 0..ROW_NUM {
                for j in 0..COL_NUM {
                    zobrist_tbl[k][i][j] = rng.gen::<ZobristKeyType>();
                }
            }
        }

        zobrist_tbl
    }

    pub fn new() -> Self {
        let mut board = Self {
            chesses: [[EMPTY_CHESS; COL_NUM]; ROW_NUM],
            role: RED,
            in_den: RoleType::EMPTY,
            zobrist_tbl: Self::gen_zobrist_tbl(),
            zobrist_key: 0,
            dup_counter: HashMap::new(),
            red_chess_num: 0,
            black_chess_num: 0,
            ctx: Vec::new(),
        };

        board.load_fen("l5t/1d3c1/r1p1w1e/7/7/7/E1W1P1R/1C3D1/T5L w");
        board
    }

    pub fn move_chess(&mut self, mv: MOVE) {
        let (src, dst) = get_move(mv);
        let eated = self.chesses[dst.0][dst.1];
        let src_chess = self.chesses[src.0][src.1];
        let zobrist_key = self.zobrist_key;

        if eated != EMPTY_CHESS {
            self.zobrist_key ^= self.zobrist_tbl[eated.get_chess_idx()][dst.0][dst.1];
        }
        self.zobrist_key ^= self.zobrist_tbl[src_chess.get_chess_idx()][dst.0][dst.1];
        self.zobrist_key ^= self.zobrist_tbl[src_chess.get_chess_idx()][src.0][src.1];

        self.chesses[dst.0][dst.1] = src_chess;
        self.chesses[src.0][src.1] = EMPTY_CHESS;

        self.in_den = self.check_in_den(get_dst_pos(mv));
        self.update_chess_num(eated, UpdateChess::DEC);

        self.switch_player();

        self.ctx.push(Context::new(eated, zobrist_key, mv));

        *self.dup_counter.entry(zobrist_key).or_insert(0) += 1;
    }

    pub fn undo_move(&mut self) {
        if let Some(context) = self.ctx.pop() {
            let (src, dst) = get_move(context.mv);
            self.chesses[src.0][src.1] = self.chesses[dst.0][dst.1];
            self.chesses[dst.0][dst.1] = context.eated;

            self.in_den = RoleType::EMPTY;
            *self.dup_counter.get_mut(&context.zobrist_key).expect("expect zobrist_key!") -= 1;
            self.zobrist_key = context.zobrist_key;
            self.update_chess_num(context.eated, UpdateChess::ADD);
            self.switch_player();
        }
    }

    fn switch_player(&mut self) {
        self.role = if self.role == RED { BLACK }
                    else { RED };
    }

    fn pos_to_idx(pos: POS) -> usize {
        let pos = get_pos(pos);
        pos.0 * COL_NUM + pos.1
    }

    fn check_at_bank(pos: POS) -> bool {
        const BANK: u64 = 0xda4c992d8000;
        BANK & (1 << Self::pos_to_idx(pos)) > 0
    }

    fn check_in_den(&self, pos: POS) -> RoleType {
        let pos_ = get_pos(pos);
        match (self.chesses[pos_.0][pos_.1].role, pos) {
            (RED, BLACK_DEN) => RED,
            (BLACK, RED_DEN) => BLACK,
            _ => { RoleType::EMPTY }
        }
    }

    fn check_in_traps(&self, pos: POS) -> bool {
        let pos_ = get_pos(pos);
        if TRAP & (1 << Self::pos_to_idx(pos)) > 0 {
            if self.chesses[pos_.0][pos_.1].role == RED {
                return pos_.0 <= 1;
            } else {
                return pos_.0 >= 7;
            }
        }

        false
    }

    fn check_in_water(pos: POS) -> bool {
        let pos = get_pos(pos);
        pos.0 >= 3 && pos.0 <= 5 && pos.1 % 3 != 0
    }

    fn check_rat(&self, src: POS, dst: POS) -> bool {
        let (src, dst) = (get_pos(src), get_pos(dst));
        if src.0 == dst.0 {
            for j in src.1.min(dst.1) ..= src.1.max(dst.1) {
                if self.chesses[src.0][j].kind == RAT && Self::check_in_water(to_pos(&(src.0, j))) {
                    return true;
                }
            }
        } else {
            for i in src.0.min(dst.0) ..= src.0.max(dst.0) {
                if self.chesses[i][src.1].kind == RAT && Self::check_in_water(to_pos(&(i, src.1))) {
                    return true;
                }
            }
        }
        false
    }

    fn get_src_dst_chess(&self, src: POS, dst: POS) -> (ChessId, ChessId) {
        let (src, dst) = (get_pos(src), get_pos(dst));
        (self.chesses[src.0][src.1],
         self.chesses[dst.0][dst.1])
    }

    fn check_movable(&self, src: POS, dst: POS) -> bool {
        {
            let src = get_pos(src);
            match (self.chesses[src.0][src.1].role, dst) {
                (RED, RED_DEN) | (BLACK, BLACK_DEN) => return false,
                _ => {}
            }
        }

        let (src_chess, dst_chess) = self.get_src_dst_chess(src, dst);
        if dst_chess == EMPTY_CHESS { return true; }
        if src_chess.role == dst_chess.role { return false; }

        match (src_chess.kind, dst_chess.kind) {
            (RAT, ELEPHANT) => ! Self::check_in_water(src),
            (ELEPHANT, RAT) => false,
            (s, d)          => s.get_idx() <= d.get_idx() || self.check_in_traps(dst)
        }
    }

    fn generate_basic_steps(&self, src: POS, to_water: bool) -> Vec<MOVE> {
        let src_ = get_pos(src);
        let (x, y) = (src_.0 as i8, src_.1 as i8);

        (0..4).into_iter().map(|idx| {
            to_move(&(get_pos(src), ((x + Self::DXY[idx].0) as usize, (y + Self::DXY[idx].1) as usize)))
        }).filter(|&mv| {
            let (_, dst) = get_move(mv);
            dst.0 < ROW_NUM && dst.1 < COL_NUM &&
            self.check_movable(src, get_dst_pos(mv)) &&
            (! Self::check_in_water(to_pos(&dst)) || to_water)
        }).collect()
    }

    fn generate_tl_steps(&self, src: POS) -> Vec<MOVE> {
        let mut basic_steps = self.generate_basic_steps(src, false);
        let src_ = get_pos(src);
        if Self::check_at_bank(src) {
            if (src_.0 + 2) % 4 == 0 { // up or down
                basic_steps.push(to_move(&(src_, ((src_.0 + 4) % 8, src_.1))));
            } else { // left or right
                if src_.1 % 6 == 0 {
                    basic_steps.push(to_move(&(src_, (src_.0, 3))));
                } else {
                    basic_steps.push(to_move(&(src_, (src_.0, 0))));
                    basic_steps.push(to_move(&(src_, (src_.0, 6))));
                }
            }

            basic_steps = basic_steps.into_iter().filter(|&mv| {
                let (src, dst) = (get_src_pos(mv), get_dst_pos(mv));
                self.check_movable(src, dst) && !self.check_rat(src, dst)
            }).collect()
        }
        basic_steps
    }

    pub fn generate_all_steps(&self) -> Vec<MOVE> {
        if self.check_win() != RoleType::EMPTY { return Vec::new(); }

        let mut moves = Vec::new();
        moves.reserve(32);
        for i in 0..ROW_NUM {
            for j in 0..COL_NUM {
                let chess_id = self.chesses[i][j];
                if chess_id.role != self.role { continue }
                moves.extend(self.generate_steps(to_pos(&(i, j))));
            }
        }
        moves
    }

    pub fn generate_steps(&self, pos: POS) -> Vec<MOVE> {
        let pos_ = get_pos(pos);
        match self.chesses[pos_.0][pos_.1].kind {
            RAT =>          { self.generate_basic_steps(pos, true)  }
            TIGER | LION => { self.generate_tl_steps(pos)           }
            _ =>            { self.generate_basic_steps(pos, false) }
        }
    }

    pub fn encode_move(&self, mv: MOVE) -> u8 {
        let (src, dst) = get_move(mv);
        let sign = |x: i8| {
            match x.cmp(&0) {
                Ordering::Less => { -1 },
                Ordering::Equal => { 0 },
                Ordering::Greater => { 1 }
            }
        };

        let dxy = (sign(dst.0 as i8 - src.0 as i8),
                    sign(dst.1 as i8 - src.1 as i8));
        let idx = Self::DXY.iter().position(|&dxy_| dxy_ == dxy).expect("dx * dy == 0!");

        let encode = (idx * ROW_NUM * COL_NUM + src.0 * COL_NUM + src.1) as u8;
        encode
    }

    pub fn decode_move(&self, idx: u8) -> MOVE {
        let idx = idx as usize;

        let src = ((idx / COL_NUM) % ROW_NUM, idx % COL_NUM);
        let idx = idx / COL_NUM / ROW_NUM;

        let mut dst = ( (src.0 as i8 + Self::DXY[idx].0) as usize,
                        (src.1 as i8 + Self::DXY[idx].1) as usize);

        if (self.chesses[src.0][src.1].kind == TIGER ||
            self.chesses[src.0][src.1].kind == LION) &&
            Self::check_at_bank(to_pos(&src)) {
                if (src.0 + 2) % 4 == 0 { // up or down
                    if Self::DXY[idx].0 != 0 {
                        let cond = (src.0 == 2) as usize * 2 + (Self::DXY[idx].0 > 0) as usize;
                        let cond2 = (cond % 3 == 0) as usize;
                        dst.0 = (4 * cond / 3 + 2) * cond2 +
                                dst.0 * (1 - cond2);
                    }

                } else { // left or right
                    if Self::DXY[idx].1 != 0 {
                        let cond = (src.1 % 6 == 0) as i8;
                        dst.1 = (cond * 3 + (1 - cond) * ((Self::DXY[idx].1 + 1) * 3)) as usize;
                    }
                }
            }

        to_move(&(src, dst))
    }

    pub fn encode_board(&self) -> Vec<Vec<Vec<u8>>>{
        // (18, 9, 7)
        let mut encoded = vec![vec![vec![0; COL_NUM]; ROW_NUM]; 18];
        for i in 0..ROW_NUM {
            for j in 0..COL_NUM {
                let chess_id = self.chesses[i][j];
                if self.role == BLACK { encoded[16][i][j] = 1; }
                encoded[17][i][j] = self.get_dup_count();

                if chess_id == EMPTY_CHESS { continue; }
                encoded[chess_id.get_chess_idx()][i][j] = 1;
            }
        }
        encoded
    }
}

