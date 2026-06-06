# ternary-morph

**Ternary mathematical morphology: erosion, dilation, opening, closing, skeletonization, and gradient detection on three-valued grids.**

## Background

Mathematical morphology — developed by Matheron and Serra at École des Mines de Paris in the 1960s — provides a rigorous framework for analyzing spatial structure in images and grids. Classical morphology operates on binary (foreground/background) or grayscale data. Ternary morphology extends this to three-state domains where the third state represents a qualitatively different category: not just "present" or "absent," but also "inhibited," "damaged," or "transitional."

In materials science, ternary grids arise naturally: +1 = intact, 0 = vacant, −1 = damaged. In cellular automata, three-state rules (like Brian's Brain) require morphological operations that respect the ternary structure. In ternary neural networks, activation maps are three-valued and morphological pooling can reduce spatial resolution while preserving structural information.

`ternary-morph` implements the full morphological toolkit — erosion, dilation, opening, closing, gradient, and skeletonization — on ternary grids with configurable connectivity (4-connected von Neumann or 8-connected Moore neighborhoods).

## How It Works

### Ternary Grid

The `TernaryGrid` stores values in {−1, 0, +1} on a 2D lattice:

```rust
let mut grid = TernaryGrid::new(width, height, fill_value);
grid.set(x, y, 1);  // +1 state
grid.get(x, y);      // → 1
```

### Morphological Operations

All operations target a specific ternary state (the `target` parameter):

- **Erosion** — A cell retains the target state only if *all* its neighbors share that state. Isolated cells are eroded to 0. This is the ternary analog of morphological shrinking.
- **Dilation** — A cell acquires the target state if *any* neighbor has it. This expands regions of the target state into neighboring cells.
- **Opening** (erode → dilate) — Removes small protrusions and noise while preserving the overall shape. Idempotent: `open(open(x)) = open(x)`.
- **Closing** (dilate → erode) — Fills small holes and gaps while preserving the overall shape.
- **Gradient** (dilate − erode) — Edge detection via the morphological gradient. Cells present in the dilated image but not in the eroded image form the boundary.
- **Skeleton** — Iteratively erodes while preserving connectivity. Junction points (cells with >1 target-state neighbor) are retained to maintain topological structure.

### Connectivity

- **4-connected** (von Neumann): up, down, left, right — 4 neighbors.
- **8-connected** (Moore): includes diagonals — 8 neighbors.

The connectivity parameter controls the aggressiveness of erosion/dilation. 8-connected dilation expands faster; 4-connected erosion is more conservative.

## Experimental Results

The test suite verifies:
- **Erosion removes isolated cells**: A single +1 in a 5×5 grid of 0s erodes to nothing.
- **Dilation expands**: A single +1 dilates to its 4 or 8 neighbors.
- **Opening removes noise**: A 3×3 block is preserved; an isolated pixel at (0,0) is removed.
- **Closing fills holes**: A single 0 in a 5×5 grid of +1s is filled.
- **Gradient detects edges**: A horizontal line of +1s produces edge cells at the boundary.
- **Skeleton reduces**: A cross shape skeletonizes to a thinner structure without disconnecting.
- **Idempotent opening**: `open(open(x)) = open(x)` — proven by test on a 3×3 block.
- **Negative state support**: Dilation with target = −1 works identically to +1.
- **Connectivity comparison**: 8-connected dilation expands more than 4-connected.

## Impact

Ternary morphology opens up image processing and spatial analysis for three-valued data. Rather than thresholding to binary and losing information, or treating ternary as grayscale and getting nonsensical results, `ternary-morph` respects the categorical nature of ternary states. The skeletonization algorithm is particularly notable: it preserves connectivity through junction detection, enabling topological analysis of ternary structures.

## Use Cases

1. **Materials Defect Analysis** — Use erosion to identify isolated damage sites, opening to remove noise from radiation damage maps (see `ternary-irradiate`), and skeletonization to trace damage networks.
2. **Ternary Image Processing** — Apply morphological pooling to ternary activation maps in neural networks. Opening removes sparse activations; closing fills gaps in dense regions.
3. **Cellular Automata Post-Processing** — After running a three-state CA (e.g., Brian's Brain), use morphological gradient to identify wavefronts and skeletonization to trace stable structures.
4. **Spatial Ternary Logic** — Combine morphological operations with ternary logic to build spatial reasoning systems: "erode the positive region, then check if any negative region remains connected."

## Open Questions

1. **Grayscale extension** — Can these operations be generalized to balanced ternary with multiple digits (e.g., values in {−3, −2, −1, 0, 1, 2, 3})?
2. **3D morphology** — The current implementation is 2D. Materials simulations often require 3D lattice morphology. Would the same algorithms extend naturally?
3. **Structuring elements** — Currently uses the default von Neumann/Moore neighborhoods. Arbitrary structuring elements would enable directionally-biased morphology.

## Connection to Oxide Stack

`ternary-morph` operates on the `TernaryGrid` abstraction from `ternary-core` and produces outputs consumable by `ternary-irradiate` (defect boundary detection), `ternary-signals` (spatial frequency analysis of morphological results), and `ternary-walk` (random walks on skeletonized structures). It is the spatial processing layer: where ternary data has geometric structure, `ternary-morph` provides the tools to analyze and transform it.
