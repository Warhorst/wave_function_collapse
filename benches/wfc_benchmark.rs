use crate::Tile::*;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use wave_function_collapse::{WfcBuilder, constraints::PossibleNeighbours};

#[derive(Copy, Clone, Eq, PartialEq)]
enum Tile {
    Water,
    Sand,
    Forest,
}

pub fn simple(c: &mut Criterion) {
    c.bench_function("simple", |b| {
        b.iter(|| {
            let tiles = vec![Water, Sand, Forest];
            let possible_neighbours = PossibleNeighbours::new(
                [
                    (Water, Water),
                    (Water, Sand),
                    (Sand, Water),
                    (Sand, Sand),
                    (Sand, Forest),
                    (Forest, Forest),
                ],
                &tiles,
            );

            WfcBuilder::<3, Tile>::new(50, 50, tiles)
                .with_constraint(possible_neighbours)
                .with_seed(42)
                .build()
                .unwrap()
                .collapse()
                .unwrap();
        })
    });
}

pub fn multi_dimension(c: &mut Criterion) {
    let dimensions = [50, 75, 100, 125];

    for dim in dimensions {
        c.bench_with_input(BenchmarkId::new("multi_dimension", dim), &dim, |b, dim| {
            b.iter(|| {
                let tiles = vec![Water, Sand, Forest];
                let possible_neighbours = PossibleNeighbours::new(
                    [
                        (Water, Water),
                        (Water, Sand),
                        (Sand, Water),
                        (Sand, Sand),
                        (Sand, Forest),
                        (Forest, Forest),
                    ],
                    &tiles,
                );

                WfcBuilder::<3, Tile>::new(*dim, *dim, tiles)
                    .with_constraint(possible_neighbours)
                    .with_seed(42)
                    .build()
                    .unwrap()
                    .collapse()
                    .unwrap();
            })
        });
    }
}

criterion_group!(benches, simple, multi_dimension);
criterion_main!(benches);
