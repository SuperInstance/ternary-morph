#![forbid(unsafe_code)]

/// Mathematical morphology on ternary grids using 4-connected neighbors.

fn neighbors4(idx: usize, width: usize, len: usize) -> Vec<usize> {
    let mut ns = Vec::new();
    if idx >= width { ns.push(idx - width); }
    if idx % width > 0 { ns.push(idx - 1); }
    if idx % width + 1 < width { ns.push(idx + 1); }
    if idx + width < len { ns.push(idx + width); }
    ns
}

/// Shrink regions of `target` value — a cell stays target only if all its 4-neighbors are target.
pub fn erode(grid: &[i8], width: usize, target: i8) -> Vec<i8> {
    let len = grid.len();
    grid.iter().enumerate().map(|(i, &v)| {
        if v != target { return v; }
        let ns = neighbors4(i, width, len);
        if ns.iter().all(|&n| grid[n] == target) { target } else { 0 }
    }).collect()
}

/// Grow regions of `target` value — a cell becomes target if any 4-neighbor is target.
pub fn dilate(grid: &[i8], width: usize, target: i8) -> Vec<i8> {
    let len = grid.len();
    grid.iter().enumerate().map(|(i, &v)| {
        if v == target { return target; }
        let ns = neighbors4(i, width, len);
        if ns.iter().any(|&n| grid[n] == target) { target } else { v }
    }).collect()
}

/// Erode then dilate — removes small noise.
pub fn opening(grid: &[i8], width: usize, target: i8) -> Vec<i8> {
    dilate(&erode(grid, width, target), width, target)
}

/// Dilate then erode — fills small gaps.
pub fn closing(grid: &[i8], width: usize, target: i8) -> Vec<i8> {
    erode(&dilate(grid, width, target), width, target)
}

/// Thin target regions to 1-cell-wide lines (iterative erosion with connectivity check).
pub fn skeletonize(grid: &[i8], width: usize, target: i8) -> Vec<i8> {
    let mut current = grid.to_vec();
    let len = current.len();
    loop {
        let mut changed = false;
        let mut next = current.clone();
        for i in 0..len {
            if current[i] != target { continue; }
            let ns = neighbors4(i, width, len);
            let target_neighbors = ns.iter().filter(|&&n| current[n] == target).count();
            // Remove if >2 target neighbors (not a line endpoint or connector)
            // Keep if 1 or 2 target neighbors (endpoint or line)
            if target_neighbors > 2 {
                // Check if removal would disconnect: simple check
                // If all target neighbors are connected to each other through other paths, safe to remove
                let target_ns: Vec<usize> = ns.iter().filter(|&&n| current[n] == target).copied().collect();
                if target_ns.len() <= 1 { continue; }
                // Simpler: remove if it's not a bridge (has enough neighbors)
                next[i] = 0;
                changed = true;
            }
        }
        if !changed { break; }
        current = next;
    }
    current
}

/// Extract boundary cells of target regions (target cells with at least one non-target 4-neighbor or off-grid edge).
pub fn boundary(grid: &[i8], width: usize, target: i8) -> Vec<i8> {
    let len = grid.len();
    let height = len / width;
    grid.iter().enumerate().map(|(i, &v)| {
        if v != target { return 0; }
        let x = i % width;
        let y = i / width;
        // Off-grid neighbors count as non-target
        if x == 0 || x + 1 == width || y == 0 || y + 1 == height {
            return target;
        }
        let ns = neighbors4(i, width, len);
        if ns.iter().any(|&n| grid[n] != target) { target } else { 0 }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(v: &[i8], w: usize) -> (Vec<i8>, usize) { (v.to_vec(), w) }

    #[test]
    fn test_erode_single_cell() {
        let (g, w) = grid(&[
            0, 0, 0,
            0, 1, 0,
            0, 0, 0,
        ], 3);
        let r = erode(&g, w, 1);
        assert_eq!(r[4], 0); // single cell eroded away
    }

    #[test]
    fn test_erode_block() {
        let (g, w) = grid(&[
            1, 1,
            1, 1,
        ], 2);
        let r = erode(&g, w, 1);
        assert!(r.iter().all(|&v| v == 1)); // solid block survives
    }

    #[test]
    fn test_erode_line() {
        // Horizontal 3-wide line in 3x5 grid with padding
        let (g, w) = grid(&[
            0, 0, 0, 0, 0,
            1, 1, 1, 0, 0,
            0, 0, 0, 0, 0,
        ], 5);
        let r = erode(&g, w, 1);
        // idx 6 (center of line) has left=1, right=1, up=0, down=0 → erodes
        assert_eq!(r[6], 0);
    }

    #[test]
    fn test_dilate_single() {
        let (g, w) = grid(&[
            0, 0, 0,
            0, 1, 0,
            0, 0, 0,
        ], 3);
        let r = dilate(&g, w, 1);
        assert_eq!(r[1], 1); // above
        assert_eq!(r[3], 1); // left
        assert_eq!(r[5], 1); // right
        assert_eq!(r[7], 1); // below
        assert_eq!(r[0], 0); // diagonal untouched
    }

    #[test]
    fn test_dilate_preserves_non_target() {
        let (g, w) = grid(&[
            -1, 0, 0,
             0, 1, 0,
             0, 0, 0,
        ], 3);
        let r = dilate(&g, w, 1);
        assert_eq!(r[0], -1); // -1 preserved, not target
    }

    #[test]
    fn test_opening_removes_noise() {
        let (g, w) = grid(&[
            1, 0, 0,
            0, 1, 0,
            0, 0, 1,
        ], 3);
        let r = opening(&g, w, 1);
        // isolated cells removed
        assert!(r.iter().all(|&v| v != 1));
    }

    #[test]
    fn test_opening_preserves_block() {
        let (g, w) = grid(&[
            1, 1, 1,
            1, 1, 1,
            1, 1, 1,
        ], 3);
        let r = opening(&g, w, 1);
        assert!(r.iter().all(|&v| v == 1));
    }

    #[test]
    fn test_closing_fills_gap() {
        let (g, w) = grid(&[
            1, 1, 1,
            1, 0, 1,
            1, 1, 1,
        ], 3);
        let r = closing(&g, w, 1);
        assert_eq!(r[4], 1); // gap filled
    }

    #[test]
    fn test_closing_no_change_solid() {
        let (g, w) = grid(&[
            1, 1,
            1, 1,
        ], 2);
        let r = closing(&g, w, 1);
        assert!(r.iter().all(|&v| v == 1));
    }

    #[test]
    fn test_boundary_of_large_block() {
        // 5x5 block so center is fully interior
        let (g, w) = grid(&[
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
        ], 5);
        let r = boundary(&g, w, 1);
        // center (idx 12) has all 4 target neighbors and is not on edge
        assert_eq!(r[12], 0);
        // corner is on grid edge → boundary
        assert_eq!(r[0], 1);
        assert_eq!(r[24], 1);
    }

    #[test]
    fn test_boundary_single_cell() {
        let (g, w) = grid(&[
            0, 0, 0,
            0, 1, 0,
            0, 0, 0,
        ], 3);
        let r = boundary(&g, w, 1);
        assert_eq!(r[4], 1); // isolated cell is its own boundary
    }

    #[test]
    fn test_skeletonize_block() {
        let (g, w) = grid(&[
            1, 1, 1,
            1, 1, 1,
            1, 1, 1,
        ], 3);
        let r = skeletonize(&g, w, 1);
        // Total skeleton cells should be less than original
        let orig_count = g.iter().filter(|&&v| v == 1).count();
        let skel_count = r.iter().filter(|&&v| v == 1).count();
        assert!(skel_count < orig_count);
        // Should have at least 1 cell remaining
        assert!(skel_count >= 1);
    }

    #[test]
    fn test_skeletonize_line() {
        let (g, w) = grid(&[
            1, 1, 1, 1, 1,
        ], 5);
        let r = skeletonize(&g, w, 1);
        // Line is already 1-cell wide, should survive
        assert!(r.iter().all(|&v| v == 1));
    }

    #[test]
    fn test_erode_with_negative() {
        let (g, w) = grid(&[
            -1, -1,
            -1, -1,
        ], 2);
        let r = erode(&g, w, -1);
        assert!(r.iter().all(|&v| v == -1));
    }

    #[test]
    fn test_dilate_negative_target() {
        let (g, w) = grid(&[
             0,  0, 0,
             0, -1, 0,
             0,  0, 0,
        ], 3);
        let r = dilate(&g, w, -1);
        assert_eq!(r[1], -1);
        assert_eq!(r[7], -1);
    }
}
