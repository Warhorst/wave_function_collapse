# wave_function_collapse

An implementation of the [Wave Function Collapse](https://en.wikipedia.org/wiki/Model_synthesis) algorithm. Currently only supports 2D boards.

Usage:

1. Create a `Wfc` builder using `Wfc::new_builder`
2. Modify the `Wfc` as needed using the builder
3. Finalize the build using the `build` method
4. Collapse the `Wfc` using `collapse`

This implementation uses the following core components to generate a board:

1. **Contstraints**: Given a specific cell, the constraints define what tiles are still possible based on its neighbours. The constraints are applied after a propagation.
2. **Probability Modifiers**: Given a specific cell, the probability modifiers define how likely or unlikely it is for a tile to be picked. The modifiers are applied when collapsing the cell with the minimum entropy.
3. **Tile Filters**: Given a specific cell, the tile filters define what tiles are possible from the beginning. The filters are applied when creating the initial, uncollapsed board.
