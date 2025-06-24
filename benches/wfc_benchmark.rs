use criterion::{criterion_group, criterion_main, Criterion};
use wave_function_collapse::{PossibleNeighbours, WaveFunctionCollapse};
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
        let possible_neighbours = PossibleNeighbours::new([
            (Water, Water),
            (Water, Sand),
            (Sand, Water),
            (Sand, Sand),
            (Sand, Forest),
            (Forest, Sand),
            (Forest, Forest),
        ], &tiles);

        WaveFunctionCollapse::<3, Tile>::new(
            50,
            50,
            tiles
        )
            .with_constraint(possible_neighbours)
            .with_seed(42)
            .collapse();
    }));
}

criterion_group!(benches, benchmark);
criterion_main!(benches);