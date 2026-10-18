// TODO instead of setting my weights using an iterator where I need to know the index in the
// tiles, I could use an initial probability modifier to set the weights initially, and
// then some to update per cell

use pad::position::Position;

/// Modifies the probability of tiles. The modifiers are applied when a tile to
/// collapse should be picked.
pub trait ProbabilityModifier<T> {
    fn modify_probability<'a>(
        &self,
        position: Position,
        width: usize,
        height: usize,
        tiles: &[T],
        collapsed_positions: impl IntoIterator<Item=(&'a T, Position)>
    ) -> impl Iterator<Item = (u8, f32)> where T: 'a;
}
