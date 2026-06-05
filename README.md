# ternary-morph

**Mathematical morphology on ternary grids. The geometry of {-1, 0, +1} shapes.**

Mathematical morphology is the math of *shape*. You take a structuring element (a small probe shape) and slide it across your image/grid. Dilation expands shapes. Erosion shrinks them. Opening removes small protrusions. Closing fills small holes. The gradient finds boundaries. Hit-or-miss finds exact patterns. Reconstruction grows a seed until it hits a wall.

This crate implements all of that for ternary grids. The key insight: in ternary, morphology operates on *one* state at a time. Dilation expands +1 into non-+1 regions. Erosion removes +1 cells that lack full +1 neighborhoods. The -1 state is preserved — morphology doesn't destroy it, just grows or shrinks the +1 regions around it. This makes ternary morphology a *selective* operation: you shape the positive without touching the negative.

The `reconstruct` function is the crown jewel: give it a marker (a seed) and a mask (a boundary), and it grows the seed until it exactly fills the masked region. No more, no less. This is connected component extraction, region growing, and flood fill — all in one operation.

## What's Inside

- **`se_3x3()`** — the standard 8-connected structuring element
- **`dilation(grid, w, h, se)`** — expand +1 regions by one SE radius
- **`erosion(grid, w, h, se)`** — shrink +1 regions by one SE radius
- **`opening(grid, w, h, se)`** — erosion then dilation. Removes small +1 noise
- **`closing(grid, w, h, se)`** — dilation then erosion. Fills small -1 holes
- **`gradient(grid, w, h, se)`** — dilation minus erosion. The *boundary* of +1 regions
- **`top_hat(grid, w, h, se)`** — original minus opening. Small bright features that disappear
- **`black_hat(grid, w, h, se)`** — closing minus original. Small dark features that get filled
- **`hit_or_miss(grid, w, h, fg_se, bg_se)`** — exact pattern matching with foreground/background SE
- **`conditional_dilation(marker, mask, w, h, se)`** — dilate only where mask is +1
- **`reconstruct(marker, mask, w, h)`** — iterative conditional dilation until stable. Flood fill

## Quick Example

```rust
use ternary_morph::*;

let se = se_3x3();

// A single +1 cell in a 5x5 grid
let grid = vec![
    0, 0, 0, 0, 0,
    0, 0, 0, 0, 0,
    0, 0, 1, 0, 0,
    0, 0, 0, 0, 0,
    0, 0, 0, 0, 0,
];

// Dilation: +1 spreads to all 8 neighbors
let dilated = dilation(&grid, 5, 5, &se);
// 3x3 block of +1 centered at (2,2)

// Erosion: the single +1 cell vanishes (not all neighbors are +1)
let eroded = erosion(&grid, 5, 5, &se);
// All zeros — isolated cells erode away

// Opening removes isolated noise
let opened = opening(&grid, 5, 5, &se);
// All zeros — the single cell was noise

// Reconstruction: grow a seed to fill a mask
let marker = vec![0,0,0, 0,1,0, 0,0,0]; // seed at center
let mask   = vec![1,1,1, 1,1,1, 1,1,1]; // allow entire 3x3
let filled = reconstruct(&marker, &mask, 3, 3);
// All +1 — the seed grew to fill the mask exactly
```

## The Deeper Truth

**Reconstruction is the most powerful operation.** All the others — dilation, erosion, opening, closing — are special cases or building blocks. Reconstruction does something unique: it's *idempotent* (running it again changes nothing) and *increasing* (it only adds +1, never removes). These properties make it a *morphological filter* — it preserves the topology of the mask while selecting regions connected to the marker. In image processing, this is how you extract individual objects from a binary image. In ternary, it's how you grow a seed population until it hits a barrier.

The `no_std` implementation (using `alloc`) means this runs on embedded systems. The ternary constraint means every intermediate result is a valid ternary grid — no floating-point, no approximation, no rounding errors. The morphology is *exact*.

**Use cases:**
- **Image processing** — ternary image segmentation, boundary detection, noise removal
- **Spatial analysis** — region growing on discrete maps
- **Game AI** — compute influence regions, fog of war, territory control
- **Document analysis** — ternary OCR (text/background/margin)
- **Embedded vision** — `no_std` morphology on microcontrollers

## See Also

- **ternary-field** — gradient and Laplacian (continuous approach to the same problems)
- **ternary-lattice** — order theory underlying morphological operations
- **ternary-diff** — diff operations on ternary grids
- **ternary-shield** — containment (related to constrained dilation)

## Install

```bash
cargo add ternary-morph
```

## License

MIT
