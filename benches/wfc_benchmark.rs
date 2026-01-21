use std::fmt::Display;

use crate::Tile::*;
use CellType::*;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use wave_function_collapse::{
    WfcBuilder,
    cell::{BitCell, ConstCell, DynCell},
    constraints::PossibleNeighbours,
};

// TODO add a benchmark without constraints for a worst case scenario
// TODO adda benchmark with a very collapsable set of tiles, as a best case scenario

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
                    Const => WfcBuilder::<Tile, ConstCell<3>>::new(50, 50, tiles)
                        .with_constraint(possible_neighbours)
                        .with_seed(42)
                        .build()
                        .unwrap()
                        .collapse()
                        .unwrap(),
                    Bit => WfcBuilder::<Tile, BitCell>::new(50, 50, tiles)
                        .with_constraint(possible_neighbours)
                        .with_seed(42)
                        .build()
                        .unwrap()
                        .collapse()
                        .unwrap(),
                    Dyn => WfcBuilder::<Tile, DynCell>::new(50, 50, tiles)
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

                WfcBuilder::<Tile, ConstCell<3>>::new(*dim, *dim, tiles)
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
