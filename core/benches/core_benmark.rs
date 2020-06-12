use criterion::{black_box, criterion_group, criterion_main, Criterion};
use animal_chess_core::board::*;
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

#[inline]
fn self_play() -> usize {
    let mut board = Board::new();
    let mut step = 0;
    loop {
        let steps = board.generate_all_steps();
        if steps.is_empty() { break; }
        board.move_chess(*steps.choose(&mut rand::thread_rng()).unwrap());
        step += 1;
    }

    step
}

fn self_play_benmark(c: &mut Criterion) {
    let mut steps = 0;
    let mut count = 0;
    c.bench_function("self_play_benmark", |b| {
        b.iter(|| {
            steps += self_play();
            count += 1;
        });
    });
    println!("average {}/{}={} steps to end", steps, count, steps / count);
}

criterion_group!(benches, gen_and_move_benmark, self_play_benmark);
criterion_main!(benches);
