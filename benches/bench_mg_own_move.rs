use std::path::Path;
use abalone::player;
use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let board = [
        [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
        [3, 3, 3, 3, 3, 1, 1, 0, 2, 2, 3],
        [3, 3, 3, 3, 1, 1, 1, 2, 2, 2, 3],
        [3, 3, 3, 0, 1, 1, 0, 2, 2, 0, 3],
        [3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 3],
        [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3],
        [3, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3],
        [3, 0, 2, 2, 0, 1, 1, 0, 3, 3, 3],
        [3, 2, 2, 2, 1, 1, 1, 3, 3, 3, 3],
        [3, 2, 2, 0, 1, 1, 3, 3, 3, 3, 3],
        [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
    ];
    let model_path = Path::new("$CARGO_MANIFEST_PATH").join("src").join("magister_zero_unwrap_save");
    let model_path_str = model_path.to_str().unwrap();
    let mut magi_ludi = player::MagisterLudi::new(board, model_path_str, 200, 10, 7, 13);
    c.bench_function("test magister own_move", |b| {
        b.iter(|| magi_ludi.own_move())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
