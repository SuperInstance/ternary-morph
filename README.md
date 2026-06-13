# ternary-morph

Ternary mathematical morphology on 2D grids {-1, 0, +1}. Erosion, dilation, opening, closing, gradient edge detection, and skeletonization with tri-state connectivity.

## Why It Matters

Classical binary morphology (Serra, 1982) operates on {0, 1} images. Ternary morphology extends this to three states, enabling operations on signed signals where -1 and +1 represent opposing polarities and 0 is the neutral background. This is essential for:
- **Ternary sensor fusion**: combining agents with positive/negative evidence
- **Signed edge detection**: boundaries between antagonistic regions
- **Ternary image processing**: tri-level quantized neural network activations
- **Spatial conservation analysis**: measuring how {-1, 0, +1} fields evolve

## How It Works

### Structuring Element Operations

Each operation uses a connectivity structuring element $\mathcal{S}$ — either 4-connected (von Neumann) or 8-connected (Moore) neighborhood:

$$\mathcal{S}_4 = \{(\pm1,0),(0,\pm1)\}, \quad \mathcal{S}_8 = \mathcal{S}_4 \cup \{(\pm1,\pm1)\}$$

### Erosion

A cell retains target value $t$ only if **all** neighbors in $\mathcal{S}$ also equal $t$:

$$\text{erode}(x) = \begin{cases} t & \text{if } \forall n \in \mathcal{S}(x): n = t \\ 0 & \text{otherwise} \end{cases}$$

Removes isolated pixels and shrinks boundaries. **Complexity:** O(W × H × |S|) where |S| ∈ {4, 8}.

### Dilation

A cell becomes target $t$ if **any** neighbor equals $t$:

$$\text{dilate}(x) = \begin{cases} t & \text{if } \exists n \in \mathcal{S}(x): n = t \\ x & \text{otherwise} \end{cases}$$

Expands regions. **Complexity:** O(W × H × |S|).

### Opening and Closing

$$\text{open}(G) = \text{dilate}(\text{erode}(G))$$
$$\text{close}(G) = \text{erode}(\text{dilate}(G))$$

Opening removes small protrusions (< |S| radius). Closing fills small holes. Both are **idempotent**: applying twice equals applying once.

### Morphological Gradient

$$\text{gradient}(G) = \text{dilate}(G) \setminus \text{erode}(G)$$

Detects boundaries — the ring of cells where dilation added $t$ but erosion did not preserve it.

### Skeletonization

Iteratively thin target regions while preserving connectivity:

1. Compute erosion
2. Remove cells whose erosion dropped them, unless they are junctions (≥2 target neighbors)
3. Repeat until stable

**Convergence:** at most $O(\min(W,H))$ iterations for a filled rectangle.

## Quick Start

```rust
use ternary_morph::*;

let mut grid = TernaryGrid::new(7, 7, 0);
// Draw a 3×3 block of +1
for y in 2..=4 { for x in 2..=4 { grid.set(x, y, 1); } }
// Add noise pixel
grid.set(0, 0, 1);

let opened = grid.open(1, 4);
assert_eq!(opened.get(0, 0), 0); // noise removed
assert_eq!(opened.get(3, 3), 1); // block preserved

let edges = grid.gradient(1, 8);
// Edge ring surrounds the block

let mut neg_grid = TernaryGrid::new(5, 5, 0);
neg_grid.set(2, 2, -1);
let dilated_neg = neg_grid.dilate(-1, 4); // works on -1 too
```

## API

| Method | Description |
|---|---|
| `TernaryGrid::new(w, h, fill)` | Create grid filled with a ternary value |
| `TernaryGrid::from_vec(w, h, data)` | Create from raw `Vec<i8>` |
| `.get(x, y) / .set(x, y, v)` | Cell access |
| `.erode(target, connectivity)` | Erode target regions |
| `.dilate(target, connectivity)` | Dilate target regions |
| `.open(target, connectivity)` | Opening (remove noise) |
| `.close(target, connectivity)` | Closing (fill holes) |
| `.gradient(target, connectivity)` | Morphological edge detection |
| `.skeleton(target)` | Thinning to skeleton |
| `.count(state)` | Count cells of given state |

## Architecture Notes

Ternary morphology directly embodies the **γ + η = C** conservation framework. The +1 cells represent constructive mass γ, -1 cells represent inhibitory mass η, and 0 cells are the neutral substrate. Morphological operations transform the *spatial arrangement* of γ and η without altering their counts (for opening/closing with matching structuring elements). The gradient operator identifies the boundary interface where γ and η meet — the domain wall structure that governs energy transfer between the two populations.

The skeletonization algorithm preserves topological connectivity by checking junction nodes, ensuring that the structural relationship between γ-regions and η-regions is maintained even as the neutral buffer zone changes shape.

## References

- Serra, J. (1982). *Image Analysis and Mathematical Morphology.* Academic Press.
- Soille, P. (2003). *Morphological Image Analysis.* Springer.
- Gonzalez, R. C. & Woods, R. E. (2018). *Digital Image Processing.* 4th ed. Pearson.
- Heijmans, H. J. A. M. (1994). *Morphological Image Operators.* Academic Press.

## License

MIT
