#![forbid(unsafe_code)]
#![no_std]
extern crate alloc;

use alloc::{vec, vec::Vec};

/// Returns the 8-connected offsets for a 3×3 structuring element (excluding center).
pub fn se_3x3() -> Vec<(i32, i32)> {
    vec![
        (-1, -1), (-1, 0), (-1, 1),
        (0, -1),           (0, 1),
        (1, -1),  (1, 0),  (1, 1),
    ]
}

/// Convert (row, col) offset to flat index, returning None if out of bounds.
fn offset_to_idx(row: i32, col: i32, r: i32, c: i32, w: usize, h: usize) -> Option<usize> {
    let nr = r + row;
    let nc = c + col;
    if nr < 0 || nc < 0 || nr >= h as i32 || nc >= w as i32 {
        None
    } else {
        Some((nr as usize) * w + (nc as usize))
    }
}

/// Dilation: a cell becomes +1 if any neighbor in the SE is +1. Non-+1 cells that
/// aren't hit remain unchanged (preserving -1 cells).
pub fn dilation(grid: &[i8], w: usize, h: usize, se: &[(i32, i32)]) -> Vec<i8> {
    let mut out = Vec::from_iter(grid.iter().copied());
    for r in 0..h as i32 {
        for c in 0..w as i32 {
            let idx = (r as usize) * w + (c as usize);
            if grid[idx] == 1 {
                continue; // already +1
            }
            for &(dr, dc) in se {
                if let Some(ni) = offset_to_idx(dr, dc, r, c, w, h) {
                    if grid[ni] == 1 {
                        out[idx] = 1;
                        break;
                    }
                }
            }
        }
    }
    out
}

/// Erosion: a +1 cell stays +1 only if ALL SE neighbors are +1.
/// Non-+1 cells remain unchanged.
pub fn erosion(grid: &[i8], w: usize, h: usize, se: &[(i32, i32)]) -> Vec<i8> {
    let mut out = Vec::from_iter(grid.iter().copied());
    for r in 0..h as i32 {
        for c in 0..w as i32 {
            let idx = (r as usize) * w + (c as usize);
            if grid[idx] != 1 {
                continue;
            }
            let all_one = se.iter().all(|&(dr, dc)| {
                offset_to_idx(dr, dc, r, c, w, h)
                    .map(|ni| grid[ni] == 1)
                    .unwrap_or(false)
            });
            if !all_one {
                out[idx] = 0;
            }
        }
    }
    out
}

/// Opening: erosion followed by dilation.
pub fn opening(grid: &[i8], w: usize, h: usize, se: &[(i32, i32)]) -> Vec<i8> {
    let eroded = erosion(grid, w, h, se);
    dilation(&eroded, w, h, se)
}

/// Closing: dilation followed by erosion.
pub fn closing(grid: &[i8], w: usize, h: usize, se: &[(i32, i32)]) -> Vec<i8> {
    let dilated = dilation(grid, w, h, se);
    erosion(&dilated, w, h, se)
}

/// Morphological gradient: dilation - erosion (elementwise).
/// Result values are in {-1, 0, +1} — clamped.
pub fn gradient(grid: &[i8], w: usize, h: usize, se: &[(i32, i32)]) -> Vec<i8> {
    let dil = dilation(grid, w, h, se);
    let ero = erosion(grid, w, h, se);
    dil.iter().zip(ero.iter()).map(|(&d, &e)| (d - e).clamp(-1, 1)).collect()
}

/// Top hat: original - opening.
pub fn top_hat(grid: &[i8], w: usize, h: usize, se: &[(i32, i32)]) -> Vec<i8> {
    let op = opening(grid, w, h, se);
    grid.iter().zip(op.iter()).map(|(&g, &o)| (g - o).clamp(-1, 1)).collect()
}

/// Black hat: closing - original.
pub fn black_hat(grid: &[i8], w: usize, h: usize, se: &[(i32, i32)]) -> Vec<i8> {
    let cl = closing(grid, w, h, se);
    cl.iter().zip(grid.iter()).map(|(&c, &g)| (c - g).clamp(-1, 1)).collect()
}

/// Hit-or-miss: match foreground pattern (fg_se must be +1) AND background pattern
/// (bg_se must be 0 or -1) simultaneously.
pub fn hit_or_miss(
    grid: &[i8], w: usize, h: usize,
    fg_se: &[(i32, i32)], bg_se: &[(i32, i32)],
) -> Vec<i8> {
    let mut out = vec![0i8; w * h];
    for r in 0..h as i32 {
        for c in 0..w as i32 {
            let idx = (r as usize) * w + (c as usize);
            let fg_match = fg_se.iter().all(|&(dr, dc)| {
                offset_to_idx(dr, dc, r, c, w, h).map(|ni| grid[ni] == 1).unwrap_or(false)
            });
            let bg_match = bg_se.iter().all(|&(dr, dc)| {
                offset_to_idx(dr, dc, r, c, w, h).map(|ni| grid[ni] != 1).unwrap_or(false)
            });
            if fg_match && bg_match {
                out[idx] = 1;
            }
        }
    }
    out
}

/// Conditional dilation: dilate marker but only where mask is +1.
pub fn conditional_dilation(
    marker: &[i8], mask: &[i8], w: usize, h: usize, se: &[(i32, i32)],
) -> Vec<i8> {
    let mut out = Vec::from_iter(marker.iter().copied());
    for r in 0..h as i32 {
        for c in 0..w as i32 {
            let idx = (r as usize) * w + (c as usize);
            if out[idx] == 1 || mask[idx] != 1 {
                continue;
            }
            for &(dr, dc) in se {
                if let Some(ni) = offset_to_idx(dr, dc, r, c, w, h) {
                    if marker[ni] == 1 && mask[idx] == 1 {
                        out[idx] = 1;
                        break;
                    }
                }
            }
        }
    }
    out
}

/// Morphological reconstruction: iteratively dilate marker constrained by mask until stable.
pub fn reconstruct(marker: &[i8], mask: &[i8], w: usize, h: usize) -> Vec<i8> {
    let se = se_3x3();
    let mut current = Vec::from_iter(marker.iter().copied());
    loop {
        let next = conditional_dilation(&current, mask, w, h, &se);
        if next == current {
            return current;
        }
        current = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(v: &[i8], w: usize, h: usize) -> (Vec<i8>, usize, usize) {
        (v.to_vec(), w, h)
    }

    #[test]
    fn test_se_3x3_count() {
        let se = se_3x3();
        assert_eq!(se.len(), 8);
    }

    #[test]
    fn test_dilation_single_cell() {
        let (grid, w, h) = g(&[
            0, 0, 0,
            0, 1, 0,
            0, 0, 0,
        ], 3, 3);
        let se = se_3x3();
        let r = dilation(&grid, w, h, &se);
        // All 8 neighbors should become +1, center stays +1
        assert_eq!(r[0], 1);
        assert_eq!(r[1], 1);
        assert_eq!(r[2], 1);
        assert_eq!(r[3], 1);
        assert_eq!(r[4], 1);
        assert_eq!(r[5], 1);
        assert_eq!(r[6], 1);
        assert_eq!(r[7], 1);
        assert_eq!(r[8], 1);
    }

    #[test]
    fn test_dilation_preserves_negative_far() {
        // -1 far from any +1 should be preserved
        let (grid, w, h) = g(&[
             0,  0, 0, 0, 0,
             0,  0, 0, 0, 0,
             0,  0, 1, 0, 0,
             0,  0, 0, 0, 0,
             0, -1, 0, 0, 0,
        ], 5, 5);
        let se = se_3x3();
        let r = dilation(&grid, w, h, &se);
        // idx 21 (row4,col1) is distance 2 from the +1 at idx 12 — not in 8-neighbor range
        assert_eq!(r[21], -1); // -1 preserved (too far)
        assert_eq!(r[16], 1);  // row3,col1 is neighbor of +1
    }

    #[test]
    fn test_erosion_single_cell_erodes() {
        let (grid, w, h) = g(&[
            0, 0, 0,
            0, 1, 0,
            0, 0, 0,
        ], 3, 3);
        let se = se_3x3();
        let r = erosion(&grid, w, h, &se);
        assert_eq!(r[4], 0); // single isolated cell erodes away
    }

    #[test]
    fn test_erosion_solid_block_center_survives() {
        // 5x5 solid block — center should survive erosion with se_3x3
        let (grid, w, h) = g(&[
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
        ], 5, 5);
        let se = se_3x3();
        let r = erosion(&grid, w, h, &se);
        // Center (idx 12) has all 8 neighbors as +1
        assert_eq!(r[12], 1);
        // Border cells erode (out-of-bounds SE neighbors)
        assert_eq!(r[0], 0); // corner
        assert_eq!(r[1], 0); // edge
    }

    #[test]
    fn test_erosion_preserves_negative() {
        let (grid, w, h) = g(&[
            -1, 0, 0,
             0, 0, 0,
             0, 0, 0,
        ], 3, 3);
        let se = se_3x3();
        let r = erosion(&grid, w, h, &se);
        assert_eq!(r[0], -1); // -1 untouched by erosion
    }

    #[test]
    fn test_opening_removes_isolated() {
        let (grid, w, h) = g(&[
            0, 0, 0,
            0, 1, 0,
            0, 0, 0,
        ], 3, 3);
        let se = se_3x3();
        let r = opening(&grid, w, h, &se);
        assert!(r.iter().all(|&v| v != 1));
    }

    #[test]
    fn test_opening_preserves_large_block() {
        let (grid, w, h) = g(&[
            1, 1, 1,
            1, 1, 1,
            1, 1, 1,
        ], 3, 3);
        let se = se_3x3();
        let r = opening(&grid, w, h, &se);
        assert!(r.iter().all(|&v| v == 1));
    }

    #[test]
    fn test_closing_fills_small_gap() {
        let (grid, w, h) = g(&[
            1, 1, 1,
            1, 0, 1,
            1, 1, 1,
        ], 3, 3);
        let se = se_3x3();
        let r = closing(&grid, w, h, &se);
        assert_eq!(r[4], 1); // gap filled
    }

    #[test]
    fn test_closing_preserves_center() {
        // 5x5 solid block — closing should not alter center region
        let (grid, w, h) = g(&[
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
        ], 5, 5);
        let se = se_3x3();
        let r = closing(&grid, w, h, &se);
        // Interior cells should be preserved by closing
        assert_eq!(r[12], 1);
    }

    #[test]
    fn test_gradient_finds_boundary() {
        let (grid, w, h) = g(&[
            0, 0, 0,
            0, 1, 0,
            0, 0, 0,
        ], 3, 3);
        let se = se_3x3();
        let r = gradient(&grid, w, h, &se);
        // Center erodes to 0, dilates to all 1s → gradient is 1 everywhere except center
        assert_eq!(r[4], 1); // dilation=1, erosion=0 → 1
    }

    #[test]
    fn test_gradient_finds_border() {
        // 5x5 solid block — gradient should be non-zero at borders, zero at center
        let (grid, w, h) = g(&[
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
        ], 5, 5);
        let se = se_3x3();
        let r = gradient(&grid, w, h, &se);
        // Center: both dilation and erosion yield +1 → gradient = 0
        assert_eq!(r[12], 0);
        // Corner: dilation = 1, erosion = 0 → gradient = 1
        assert_eq!(r[0], 1);
    }

    #[test]
    fn test_top_hat_isolated_peak() {
        let (grid, w, h) = g(&[
            0, 0, 0,
            0, 1, 0,
            0, 0, 0,
        ], 3, 3);
        let se = se_3x3();
        let r = top_hat(&grid, w, h, &se);
        // Opening removes the isolated +1, so top_hat = orig - opening = 1 at center
        assert_eq!(r[4], 1);
    }

    #[test]
    fn test_black_hat_fills_gap() {
        let (grid, w, h) = g(&[
            1, 1, 1,
            1, 0, 1,
            1, 1, 1,
        ], 3, 3);
        let se = se_3x3();
        let r = black_hat(&grid, w, h, &se);
        // Closing fills the gap → black_hat = closing - orig → 1 at center
        assert_eq!(r[4], 1);
    }

    #[test]
    fn test_hit_or_miss_corner() {
        let (grid, w, h) = g(&[
            1, 0, 0,
            0, 0, 0,
            0, 0, 0,
        ], 3, 3);
        // Looking for a +1 cell with right=0 and below=0
        let fg_se: Vec<(i32, i32)> = vec![(0, 0)]; // center must be +1
        let bg_se: Vec<(i32, i32)> = vec![(0, 1), (1, 0)]; // right and below must not be +1
        let r = hit_or_miss(&grid, w, h, &fg_se, &bg_se);
        assert_eq!(r[0], 1); // top-left corner matches
    }

    #[test]
    fn test_hit_or_miss_no_match() {
        let (grid, w, h) = g(&[
            1, 1, 0,
            1, 0, 0,
            0, 0, 0,
        ], 3, 3);
        let fg_se: Vec<(i32, i32)> = vec![(0, 0)];
        let bg_se: Vec<(i32, i32)> = vec![(0, 1), (1, 0)]; // right and below must be non-+1
        let r = hit_or_miss(&grid, w, h, &fg_se, &bg_se);
        assert_eq!(r[0], 0); // top-left has right=1, doesn't match
    }

    #[test]
    fn test_conditional_dilation() {
        // marker: single +1 in center
        // mask: full row
        let (marker, w, h) = g(&[
            0, 0, 0,
            0, 1, 0,
            0, 0, 0,
        ], 3, 3);
        let mask = vec![
            0, 0, 0,
            1, 1, 1,
            0, 0, 0,
        ];
        let se = se_3x3();
        let r = conditional_dilation(&marker, &mask, w, h, &se);
        // Should dilate only along the masked row
        assert_eq!(r[3], 1); // left of center
        assert_eq!(r[4], 1); // center (already +1)
        assert_eq!(r[5], 1); // right of center
        assert_eq!(r[1], 0); // above, not in mask
    }

    #[test]
    fn test_reconstruct() {
        // marker: single +1 in center
        // mask: full 3x3 block
        let (marker, w, h) = g(&[
            0, 0, 0,
            0, 1, 0,
            0, 0, 0,
        ], 3, 3);
        let mask = vec![
            1, 1, 1,
            1, 1, 1,
            1, 1, 1,
        ];
        let r = reconstruct(&marker, &mask, w, h);
        // Should reconstruct entire mask
        assert!(r.iter().all(|&v| v == 1));
    }

    #[test]
    fn test_reconstruct_partial_mask() {
        // marker: single +1 in top-left
        // mask: L-shape
        let (marker, w, h) = g(&[
            1, 0, 0,
            0, 0, 0,
            0, 0, 0,
        ], 3, 3);
        let mask = vec![
            1, 1, 0,
            1, 0, 0,
            1, 0, 0,
        ];
        let r = reconstruct(&marker, &mask, w, h);
        assert_eq!(r[0], 1);
        assert_eq!(r[1], 1);
        assert_eq!(r[3], 1);
        assert_eq!(r[6], 1);
        assert_eq!(r[2], 0);
        assert_eq!(r[4], 0);
    }
}
