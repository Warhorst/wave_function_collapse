use pad::{p, position::Position};
use wave_function_collapse::{
    WeightsError, Wfc, WfcError,
    board::BoardError,
    cell::{DynCell, PossibleIndices},
    tile_filter::{FilteredTiles, TileFilterContext},
};

#[test]
fn all_positions_collapsed() {
    let width = 5;
    let height = 5;
    let wfc = default_wfc(width, height);

    let result = wfc.collapse().unwrap();

    assert_eq!(result.len(), width * height);

    for y in 0..height {
        for x in 0..width {
            let pos = p!(x, y);
            let occurrences = result.iter().filter(|(p, _)| *p == pos).count();
            assert_eq!(
                occurrences, 1,
                "position {:?} occurred {} times",
                pos, occurrences
            );
        }
    }
}

#[test]
fn invalid_weights_detected() {
    let builder = || Wfc::<i32, DynCell>::new_builder(3, 3, vec![0, 1, 2]);

    let too_few = builder().with_weights([1.0]).build().err();
    assert_eq!(
        too_few,
        Some(WfcError::WeightsError(WeightsError::InvalidAmount {
            expected: 3,
            was: 1
        }))
    );

    let too_many = builder().with_weights([1.0; 5]).build().err();
    assert_eq!(
        too_many,
        Some(WfcError::WeightsError(WeightsError::InvalidAmount {
            expected: 3,
            was: 5
        }))
    );

    let negative = builder().with_weights([1.0, -1.0, 1.0]).build().err();
    assert_eq!(
        negative,
        Some(WfcError::WeightsError(WeightsError::WeightLowerZero))
    );

    let nan = builder().with_weights([1.0, f32::NAN, 1.0]).build().err();
    assert_eq!(nan, Some(WfcError::WeightsError(WeightsError::WeightNaN)));

    let all_zero = builder().with_weights([0.0; 3]).build().err();
    assert_eq!(
        all_zero,
        Some(WfcError::WeightsError(WeightsError::AllZero))
    );
}

#[test]
fn collapse_position() {
    let pos = p!(1, 1);

    // collapse non collapsed position -> works
    let mut wfc = default_wfc(3, 3);
    let result = wfc.collapse_position(pos);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 1);

    // collapse position that is already collapsed -> fails
    let mut wfc = default_wfc(3, 3);
    wfc.collapse_position(pos).unwrap();
    assert_eq!(
        wfc.collapse_position(pos),
        Err(WfcError::PositionAlreadyCollapsed(pos))
    );

    // collapse non collapsed position with possible tile -> works
    let mut wfc = default_wfc(3, 3);
    let result = wfc.collapse_position_and_tile(pos, 1);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![(pos, 1)]);

    // collapse non collapsed position with impossible tile -> fails
    let mut wfc = default_wfc(3, 3);
    assert_eq!(
        wfc.collapse_position_and_tile(pos, 4),
        Err(WfcError::InvalidTileIndex(4, pos))
    );

    // collapse already collapsed position with possible tile -> fails
    let mut wfc = default_wfc(3, 3);
    wfc.collapse_position(pos).unwrap();
    assert_eq!(
        wfc.collapse_position_and_tile(pos, 1),
        Err(WfcError::PositionAlreadyCollapsed(pos))
    );

    // collapse already collapsed position with impossible tile -> fails
    let mut wfc = default_wfc(3, 3);
    wfc.collapse_position_and_tile(pos, 0).unwrap();
    assert_eq!(
        wfc.collapse_position_and_tile(pos, 4),
        Err(WfcError::PositionAlreadyCollapsed(pos))
    );
}

#[test]
fn board_pre_filter_zero_entropy_error() {
    let filter_all_at_origin =
        |position: Position, filtered: &mut FilteredTiles, context: TileFilterContext<i32>| {
            if position == p!(0, 0) {
                for i in 0..context.tiles.len() {
                    filtered.filter(i);
                }
            }
        };

    let result = Wfc::<i32, DynCell>::new_builder(3, 3, vec![0, 1, 2])
        .with_tile_filter(filter_all_at_origin)
        .build()
        .err();

    assert_eq!(
        result,
        Some(WfcError::BoardError(BoardError::ZeroEntropyAfterFilter(
            p!(0, 0)
        )))
    );
}

#[test]
fn board_zero_entropy_after_collapse_error() {
    let tiles = vec![0, 1, 2];
    let constraint = |_: (u8, Position), _: &[(PossibleIndices, Position)], _: &[i32]| false;

    let wfc = Wfc::<i32, DynCell>::new_builder(3, 3, tiles)
        .with_constraint(constraint)
        .build()
        .unwrap();

    let result = wfc.collapse();

    assert!(matches!(
        result,
        Err(WfcError::BoardError(BoardError::ZeroEntropyAfterPropagate(
            _
        )))
    ));
}

fn default_wfc(
    width: usize,
    height: usize,
) -> Wfc<i32, DynCell> {
    Wfc::<i32, DynCell>::new_builder(width, height, vec![0, 1, 2])
        .build()
        .unwrap()
}
