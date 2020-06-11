use criterion::{black_box, criterion_group, criterion_main, Criterion};
use animal_chess_core::board::*;
use animal_chess_core::chess::*;
use core::time::Duration;
use rand::seq::SliceRandom;

#[inline]
fn gen_and_move_chess(max_times: usize) {
    let mut board = Board::new();
    let mut times = 0;
    while times < max_times {
        if let Some(&mv) = board.generate_all_steps().first() {
            board.move_chess(mv);
        } else {
            break;
        }
        times += 1;
    }
}

fn gen_and_move_benmark(c: &mut Criterion) {
    c.bench_function("gen_and_move_benmark", |b| {
        b.iter(|| gen_and_move_chess(black_box(500)))
    });
}

criterion_group!(benches, gen_and_move_benmark);
criterion_main!(benches);
