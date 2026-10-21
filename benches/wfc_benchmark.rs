use std::fmt::Display;

use crate::Tile::*;
use CellType::*;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use wave_function_collapse::{
    Wfc,
    cell::{BitCell, ConstCell, DynCell},
    constraints::PossibleNeighbours,
};

// TODO add a benchmark with a very collapsable set of tiles, as a best case scenario

#[derive(Copy, Clone, Eq, PartialEq)]
enum Tile {
    Water,
    Sand,
    Forest,
}

#[derive(Clone, Copy, Debug)]
enum CellType {
    Const,
    Bit,
    Dyn,
}

impl Display for CellType {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl CellType {
    fn values() -> [CellType; 3] {
        [Const, Bit, Dyn]
    }
}

pub fn simple(c: &mut Criterion) {
    for cell in CellType::values() {
        c.bench_with_input(BenchmarkId::new("simple", cell), &cell, |b, cell| {
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

                match cell {
                    Const => Wfc::<Tile, ConstCell<3>>::new_builder(50, 50, tiles)
                        .with_constraint(possible_neighbours)
                        .with_seed(42)
                        .build()
                        .unwrap()
                        .collapse()
                        .unwrap(),
                    Bit => Wfc::<Tile, BitCell>::new_builder(50, 50, tiles)
                        .with_constraint(possible_neighbours)
                        .with_seed(42)
                        .build()
                        .unwrap()
                        .collapse()
                        .unwrap(),
                    Dyn => Wfc::<Tile, DynCell>::new_builder(50, 50, tiles)
                        .with_constraint(possible_neighbours)
                        .with_seed(42)
                        .build()
                        .unwrap()
                        .collapse()
                        .unwrap(),
                };
            });
        });
    }
}

pub fn no_constraints(c: &mut Criterion) {
    let dimensions = [50, 75, 100, 125, 250, 500];

    for dim in dimensions {
        c.bench_with_input(BenchmarkId::new("no_constraints", dim), &dim, |b, dim| {
            b.iter(|| {
                let tiles = vec![Water, Sand, Forest];

                Wfc::<Tile, ConstCell<3>>::new_builder(*dim, *dim, tiles)
                    .with_seed(42)
                    .build()
                    .unwrap()
                    .collapse()
                    .unwrap();
            })
        });
    }
}

pub fn possible_neighbours(c: &mut Criterion) {
    let dimensions = [50, 75, 100, 125];

    for dim in dimensions {
        c.bench_with_input(
            BenchmarkId::new("possible_neighbours", dim),
            &dim,
            |b, dim| {
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

                    Wfc::<Tile, ConstCell<3>>::new_builder(*dim, *dim, tiles)
                        .with_constraint(possible_neighbours)
                        .with_seed(42)
                        .build()
                        .unwrap()
                        .collapse()
                        .unwrap();
                })
            },
        );
    }
}

criterion_group!(benches, simple, no_constraints, possible_neighbours);
criterion_main!(benches);
