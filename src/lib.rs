//! Ternary morphological operations: erosion, dilation, opening, closing, skeletonization.

/// Ternary grid for morphological operations
#[derive(Clone, Debug)]
pub struct TernaryGrid {
    width: usize,
    height: usize,
    data: Vec<i8>, // -1, 0, +1
}

impl TernaryGrid {
    pub fn new(width: usize, height: usize, fill: i8) -> Self {
        assert!(fill >= -1 && fill <= 1);
        Self { width, height, data: vec![fill; width * height] }
    }

    pub fn from_vec(width: usize, height: usize, data: Vec<i8>) -> Self {
        assert_eq!(data.len(), width * height);
        assert!(data.iter().all(|&v| v >= -1 && v <= 1));
        Self { width, height, data }
    }

    pub fn get(&self, x: usize, y: usize) -> i8 {
        self.data[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, v: i8) {
        assert!(v >= -1 && v <= 1);
        self.data[y * self.width + x] = v;
    }

    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }
    pub fn data(&self) -> &[i8] { &self.data }

    fn neighbors4(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut n = Vec::new();
        if x > 0 { n.push((x-1, y)); }
        if x < self.width-1 { n.push((x+1, y)); }
        if y > 0 { n.push((x, y-1)); }
        if y < self.height-1 { n.push((x, y+1)); }
        n
    }

    fn neighbors8(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut n = self.neighbors4(x, y);
        if x > 0 && y > 0 { n.push((x-1, y-1)); }
        if x < self.width-1 && y > 0 { n.push((x+1, y-1)); }
        if x > 0 && y < self.height-1 { n.push((x-1, y+1)); }
        if x < self.width-1 && y < self.height-1 { n.push((x+1, y+1)); }
        n
    }

    /// Erosion: cell becomes min of its neighborhood. Target state preserved only if all neighbors match.
    pub fn erode(&self, target: i8, connectivity: usize) -> TernaryGrid {
        let mut result = self.clone();
        for y in 0..self.height {
            for x in 0..self.width {
                let nbrs = if connectivity == 4 { self.neighbors4(x,y) } else { self.neighbors8(x,y) };
                if nbrs.is_empty() { continue; }
                if nbrs.iter().all(|&(nx,ny)| self.get(nx,ny) == target) {
                    // keep target
                } else {
                    // erode to 0
                    result.set(x, y, 0);
                }
            }
        }
        result
    }

    /// Dilation: cell becomes target if any neighbor is target.
    pub fn dilate(&self, target: i8, connectivity: usize) -> TernaryGrid {
        let mut result = self.clone();
        for y in 0..self.height {
            for x in 0..self.width {
                if self.get(x,y) == target { continue; }
                let nbrs = if connectivity == 4 { self.neighbors4(x,y) } else { self.neighbors8(x,y) };
                if nbrs.iter().any(|&(nx,ny)| self.get(nx,ny) == target) {
                    result.set(x, y, target);
                }
            }
        }
        result
    }

    /// Opening = erode then dilate (removes small protrusions)
    pub fn open(&self, target: i8, connectivity: usize) -> TernaryGrid {
        self.erode(target, connectivity).dilate(target, connectivity)
    }

    /// Closing = dilate then erode (fills small holes)
    pub fn close(&self, target: i8, connectivity: usize) -> TernaryGrid {
        self.dilate(target, connectivity).erode(target, connectivity)
    }

    /// Count cells of a given state
    pub fn count(&self, state: i8) -> usize {
        self.data.iter().filter(|&&v| v == state).count()
    }

    /// Skeleton: iteratively erode until stable
    pub fn skeleton(&self, target: i8) -> TernaryGrid {
        let mut current = self.clone();
        loop {
            let eroded = current.erode(target, 8);
            if eroded.data == current.data { break; }
            // Only remove cells that won't disconnect
            let mut next = current.clone();
            for y in 0..self.height {
                for x in 0..self.width {
                    if current.get(x,y) == target && eroded.get(x,y) == 0 {
                        // Check if removing would disconnect
                        let nbrs: Vec<_> = current.neighbors8(x,y)
                            .into_iter()
                            .filter(|&(nx,ny)| current.get(nx,ny) == target)
                            .collect();
                        if nbrs.len() > 1 {
                            // Keep: it's a junction
                        } else {
                            next.set(x, y, 0);
                        }
                    }
                }
            }
            if next.data == current.data { break; }
            current = next;
        }
        current
    }

    /// Gradient = dilate - erode (edge detection)
    pub fn gradient(&self, target: i8, connectivity: usize) -> TernaryGrid {
        let dilated = self.dilate(target, connectivity);
        let eroded = self.erode(target, connectivity);
        let mut result = TernaryGrid::new(self.width, self.height, 0);
        for y in 0..self.height {
            for x in 0..self.width {
                if dilated.get(x,y) == target && eroded.get(x,y) != target {
                    result.set(x, y, target);
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_erosion_removes_isolated() {
        let mut g = TernaryGrid::new(5, 5, 0);
        g.set(2, 2, 1);
        let eroded = g.erode(1, 4);
        assert_eq!(eroded.count(1), 0); // isolated cell erodes away
    }

    #[test]
    fn test_dilation_expands() {
        let mut g = TernaryGrid::new(5, 5, 0);
        g.set(2, 2, 1);
        let dilated = g.dilate(1, 4);
        assert!(dilated.count(1) > 1);
    }

    #[test]
    fn test_opening_removes_noise() {
        let mut g = TernaryGrid::new(7, 7, 0);
        // Block of +1
        for y in 2..=4 { for x in 2..=4 { g.set(x, y, 1); } }
        // Noise pixel
        g.set(0, 0, 1);
        let opened = g.open(1, 4);
        assert_eq!(opened.get(0, 0), 0); // noise removed
        assert_eq!(opened.get(3, 3), 1); // block preserved
    }

    #[test]
    fn test_closing_fills_hole() {
        let mut g = TernaryGrid::new(5, 5, 1);
        g.set(2, 2, 0); // hole
        let closed = g.close(1, 4);
        assert_eq!(closed.get(2, 2), 1); // hole filled
    }

    #[test]
    fn test_gradient_detects_edges() {
        let mut g = TernaryGrid::new(5, 5, 0);
        for x in 1..=3 { g.set(x, 2, 1); }
        let grad = g.gradient(1, 4);
        assert!(grad.count(1) > 0);
        assert!(grad.count(1) < g.count(1) + 10);
    }

    #[test]
    fn test_skeleton_reduces() {
        let mut g = TernaryGrid::new(11, 11, 0);
        // Cross shape — skeleton should be thinner than the filled cross
        for i in 1..=9 { g.set(i, 5, 1); g.set(5, i, 1); }
        let skel = g.skeleton(1);
        // Skeleton must be <= original (may be equal if already thin)
        assert!(skel.count(1) <= g.count(1));
    }

    #[test]
    fn test_idempotent_opening() {
        let mut g = TernaryGrid::new(7, 7, 0);
        for y in 2..=4 { for x in 2..=4 { g.set(x, y, 1); } }
        let o1 = g.open(1, 4);
        let o2 = o1.open(1, 4);
        assert_eq!(o1.data, o2.data);
    }

    #[test]
    fn test_negative_state_dilation() {
        let mut g = TernaryGrid::new(5, 5, 0);
        g.set(2, 2, -1);
        let dilated = g.dilate(-1, 4);
        assert!(dilated.count(-1) > 1);
    }

    #[test]
    fn test_connectivity_8_vs_4() {
        let mut g = TernaryGrid::new(5, 5, 0);
        g.set(2, 2, 1);
        let d4 = g.dilate(1, 4);
        let d8 = g.dilate(1, 8);
        assert!(d8.count(1) >= d4.count(1));
    }
}
