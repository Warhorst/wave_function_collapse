use criterion::{criterion_group, criterion_main, Criterion};
use wave_function_collapse::{TileConstraints, wave_function_collapse};
use crate::Tile::*;

#[derive(Copy, Clone, Eq, PartialEq)]
enum Tile {
    Water,
    Sand,
    Forest
}

pub fn benchmark(c: &mut Criterion) {
    c.bench_function("wfc", |b| b.iter(|| {
        let tiles = vec![Water, Sand, Forest];
        let constraints = TileConstraints::new(
            &tiles,
            vec![
                (Water, Water),
                (Water, Sand),
                (Sand, Water),
                (Sand, Sand),
                (Sand, Forest),
                (Forest, Sand),
                (Forest, Forest),
            ],
        );

        wave_function_collapse::<3, Tile>(tiles, 50, 50, constraints, Some(42))
    }));
}

criterion_group!(benches, benchmark);
criterion_main!(benches);