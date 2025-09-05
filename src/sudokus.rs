#![allow(dead_code)] // for development, go away annoying squiggly lines

/// Bitmap of seen values
struct Seen(u16);

impl Seen {
    fn new() -> Self {
        Seen(0)
    }

    fn contains(&self, value: u8) -> bool {
        self.0 & (1 << value) > 0
    }

    fn add(&mut self, value: u8) {
        debug_assert!(value < 16);
        self.0 |= 1 << value;
    }
}

/// A row, column, or block, described by a list of its 9 indexes
pub struct Region<'a, T: Iterator<Item = usize>>(&'a SudokuGrid, T);

impl<'a, T: Iterator<Item = usize>> Region<'a, T> {
    pub fn values(self) -> impl Iterator<Item = &'a u8> {
        self.1.map(|i| &self.0.cells[i])
    }

    pub fn indexes(self) -> impl Iterator<Item = usize> {
        self.1
    }

    /// Check if region contains the numbers 1 to 9 (assuming the sequence is 9 long)
    ///
    /// Ignores empty (0) cells if partial is true
    pub fn validate(self, partial: bool) -> bool {
        let mut seen = Seen::new();
        for i in self.1 {
            let value = self.0.cells[i];
            if partial && value == 0 {
                continue;
            }
            if value == 0 || value > 9 || seen.contains(value) {
                return false;
            }
            seen.add(value);
        }
        true
    }
}

#[derive(Debug, Clone)]
pub struct Sudoku {
    pub block_start: [usize; 9],
    x: usize,
    y: usize,
}

impl Sudoku {
    fn indexes(&self) -> impl Iterator<Item = usize> + use<'_> {
        let block_order = [
            BOTTOM_LEFT_BLOCK,
            BOTTOM_RIGHT_BLOCK,
            TOP_LEFT_BLOCK,
            TOP_RIGHT_BLOCK,
            BOTTOM_CENTER_BLOCK,
            TOP_CENTER_BLOCK,
            MIDDLE_LEFT_BLOCK,
            MIDDLE_RIGHT_BLOCK,
            MIDDLE_CENTER_BLOCK, // testing showed that doing center last gives fasted solves
        ];
        block_order
            .into_iter()
            .map(|b| self.block_start[b])
            .flat_map(|b| b..(b + 9))
    }

    pub fn coords(&self) -> Coords {
        (self.x, self.y)
    }
}

// block indexes
pub const TOP_LEFT_BLOCK: usize = 0;
pub const TOP_CENTER_BLOCK: usize = 1;
pub const TOP_RIGHT_BLOCK: usize = 2;
pub const MIDDLE_LEFT_BLOCK: usize = 3;
pub const MIDDLE_CENTER_BLOCK: usize = 4;
pub const MIDDLE_RIGHT_BLOCK: usize = 5;
pub const BOTTOM_LEFT_BLOCK: usize = 6;
pub const BOTTOM_CENTER_BLOCK: usize = 7;
pub const BOTTOM_RIGHT_BLOCK: usize = 8;

pub const BLOCK_MEMORY_ORDER: [usize; 7] = [
    TOP_CENTER_BLOCK,
    MIDDLE_LEFT_BLOCK,
    MIDDLE_CENTER_BLOCK,
    MIDDLE_RIGHT_BLOCK,
    BOTTOM_LEFT_BLOCK,
    BOTTOM_CENTER_BLOCK,
    BOTTOM_RIGHT_BLOCK,
];

pub struct SudokuGrid {
    /// Stores all cells without overlap (so 7 * 9 cells per sudoku)
    pub cells: Box<[u8]>,
    sudokus: Box<[Sudoku]>,
    pub n: usize,
    pub m: usize,
}

#[derive(Debug)]
pub struct NoSolution;

pub type Coords = (usize, usize);

pub struct DfsBlock<'a> {
    indexes: Box<[usize]>,
    sudoku_coords: Coords,
    other_sudoku_coords: Option<Coords>,
    random: &'a [u8],
    value_index: [usize; 9],
    backtracks: u64,
    i: usize,
}

impl<'a> DfsBlock<'a> {
    pub fn new(sg: &SudokuGrid, sudoku_coords: Coords, i: usize, random: &'a [u8]) -> Self {
        let indexes = sg.block(sg.sudoku(sudoku_coords), i).indexes().collect();

        let other_sudoku_coords = match i {
            TOP_LEFT_BLOCK => Some((sudoku_coords.0, (sudoku_coords.1 + 1) % sg.m)),
            TOP_RIGHT_BLOCK => Some(((sudoku_coords.0 + 1) % sg.n, sudoku_coords.1)),
            BOTTOM_LEFT_BLOCK => Some(((sudoku_coords.0 + sg.n - 1) % sg.n, sudoku_coords.1)),
            BOTTOM_RIGHT_BLOCK => Some((sudoku_coords.0, (sudoku_coords.1 + sg.m - 1) % sg.m)),
            _ => None,
        };

        DfsBlock {
            indexes,
            sudoku_coords,
            other_sudoku_coords,
            value_index: [0; 9],
            random,
            backtracks: 0,
            i: 0,
        }
    }

    fn backtrack(&mut self, sg: &mut SudokuGrid) -> Result<(), NoSolution> {
        self.backtracks += 1;
        while self.value_index[self.i] >= 8 {
            sg.cells[self.indexes[self.i]] = 0;
            self.value_index[self.i] = 0;
            if self.i == 0 {
                return Err(NoSolution);
            }
            self.i -= 1;
        }
        self.value_index[self.i] += 1;

        Ok(())
    }

    pub fn next_solution(&mut self, sg: &mut SudokuGrid) -> Result<u64, NoSolution> {
        if self.i >= 9 {
            self.i = 8;
            self.value_index[self.i] += 1;
            println!("i: {}, vi: {}", self.i, self.value_index[self.i]);
            if self.value_index[self.i] >= 8 {
                self.backtrack(sg)?
            }
            println!("i: {}", self.i);
        }

        while self.i < 9 {
            let index = self.indexes[self.i];
            sg.cells[index] = self.random[self.value_index[self.i]];

            while (sg.cell_is_problematic(self.sudoku_coords, index)
                || self
                    .other_sudoku_coords
                    .is_some_and(|s| sg.cell_is_problematic(s, index)))
                && self.value_index[self.i] < 8
            {
                self.value_index[self.i] += 1;
                sg.cells[index] = self.random[self.value_index[self.i]];
            }

            if sg.cell_is_problematic(self.sudoku_coords, index)
                || self
                    .other_sudoku_coords
                    .is_some_and(|s| sg.cell_is_problematic(s, index))
            {
                // there is no solution, we should backtrack
                self.backtrack(sg)?;
            } else {
                // let's try and see if this works
                self.i += 1;
            }
        }

        Ok(self.backtracks)
    }

    pub fn reset(&mut self, sg: &mut SudokuGrid) {
        self.i = 0;
        self.backtracks = 0;
        self.value_index = [0; 9];
        for i in self.indexes.iter() {
            sg.cells[*i] = 0;
        }
    }
}

impl SudokuGrid {
    pub fn new(n: usize, m: usize) -> Self {
        let new_sudoku = |x: usize, y: usize| -> Sudoku {
            let this_start = (x + y * n) * 7 * 9;
            Sudoku {
                block_start: [
                    (x + ((y + 1) % m) * n) * 7 * 9 + 6 * 9,
                    this_start,
                    (((x + 1) % n) + y * n) * 7 * 9 + 4 * 9,
                    this_start + 9,
                    this_start + 2 * 9,
                    this_start + 3 * 9,
                    this_start + 4 * 9,
                    this_start + 5 * 9,
                    this_start + 6 * 9,
                ],
                x,
                y,
            }
        };

        SudokuGrid {
            cells: vec![0; 7 * 9 * n * m].into(),
            sudokus: (0..n * m).map(|i| new_sudoku(i % n, i / n)).collect(),
            n,
            m,
        }
    }

    pub fn sudoku(&self, coords: Coords) -> &Sudoku {
        debug_assert!(coords.0 < self.n && coords.1 < self.m);
        &self.sudokus[coords.0 + coords.1 * self.n]
    }

    /// Loop over sudoku rows (for export to send to WebGL)
    /// TODO: update WebGL shader to use the original cell data format directly?
    pub fn sudoku_rows(&self) -> impl Iterator<Item = u8> + use<'_> {
        let x = 0..self.n;
        let y = (0..self.m).rev();
        y.flat_map(move |y| x.clone().map(move |x| self.sudoku((x, y))))
            .flat_map(move |s| (0..9).flat_map(move |y| self.row(s, y).values().copied()))
    }

    pub fn block(&self, sudoku: &Sudoku, i: usize) -> Region<'_, impl Iterator<Item = usize>> {
        debug_assert!(i < 9, "block index out of bounds");
        let start = sudoku.block_start[i];
        Region(self, start..(start + 9))
    }

    pub fn row(&self, sudoku: &Sudoku, y: usize) -> Region<'_, impl Iterator<Item = usize>> {
        debug_assert!(y < 9, "row index out of bounds");
        let first_block = (y / 3) * 3;
        let blocks = [
            sudoku.block_start[first_block],
            sudoku.block_start[first_block + 1],
            sudoku.block_start[first_block + 2],
        ];
        let first_offset = (y % 3) * 3;
        let offsets = [first_offset, first_offset + 1, first_offset + 2];
        Region(
            self,
            blocks
                .into_iter()
                .flat_map(move |b| offsets.into_iter().map(move |o| o + b)),
        )
    }

    pub fn column(&self, sudoku: &Sudoku, x: usize) -> Region<'_, impl Iterator<Item = usize>> {
        debug_assert!(x < 9, "column index out of bounds");
        let first_block = x / 3;
        let blocks = [
            sudoku.block_start[first_block],
            sudoku.block_start[first_block + 3],
            sudoku.block_start[first_block + 6],
        ];
        let first_offset = x % 3;
        let offsets = [first_offset, first_offset + 3, first_offset + 6];
        Region(
            self,
            blocks
                .into_iter()
                .flat_map(move |b| offsets.into_iter().map(move |o| o + b)),
        )
    }

    pub fn block_index_for(&self, sudoku: &Sudoku, i: usize) -> usize {
        for (block, start) in sudoku.block_start.iter().enumerate() {
            if i >= *start && i < start + 9 {
                return block;
            }
        }
        panic!("Index {i} not found in sudoku {sudoku:?}");
    }

    /// Get row for cell index
    pub fn row_for(&self, sudoku: &Sudoku, i: usize) -> Region<'_, impl Iterator<Item = usize>> {
        let block = self.block_index_for(sudoku, i);
        let row = (i % 9) / 3 + block / 3 * 3;
        self.row(sudoku, row)
    }

    /// Get column for cell index
    pub fn column_for(&self, sudoku: &Sudoku, i: usize) -> Region<'_, impl Iterator<Item = usize>> {
        let block = self.block_index_for(sudoku, i);
        let column = i % 3 + (block % 3) * 3;
        self.column(sudoku, column)
    }

    /// Get block for cell index
    pub fn block_for(&self, sudoku: &Sudoku, i: usize) -> Region<'_, impl Iterator<Item = usize>> {
        self.block(sudoku, self.block_index_for(sudoku, i))
    }

    pub fn set_block(&mut self, sudoku: &Sudoku, i: usize, values: impl Iterator<Item = u8>) {
        for (i, v) in self.block(sudoku, i).indexes().zip(values) {
            self.cells[i] = v;
        }
    }

    /// Check if sudoku is solved correctly
    pub fn is_solved(&self, sudoku: &Sudoku) -> bool {
        // row constraint
        if !(0..9).all(|i| self.row(sudoku, i).validate(false)) {
            return false;
        }

        // column constraint
        if !(0..9).all(|i| self.column(sudoku, i).validate(false)) {
            return false;
        }

        // block constraint
        if !(0..9).all(|i| self.block(sudoku, i).validate(false)) {
            return false;
        }

        true
    }

    pub fn is_solved_all(&self) -> bool {
        for x in 0..self.n {
            for y in 0..self.m {
                if !self.is_solved(self.sudoku((x, y))) {
                    return false;
                }
            }
        }
        true
    }

    /// Check if the cell at index i is problematic
    pub fn cell_is_problematic(&self, sudoku_coords: Coords, i: usize) -> bool {
        let sudoku = self.sudoku(sudoku_coords);
        !self.row_for(sudoku, i).validate(true)
            || !self.column_for(sudoku, i).validate(true)
            || !self.block_for(sudoku, i).validate(true)
    }

    pub fn sudoku_at_index(&self, i: usize) -> Coords {
        let sudoku_i = i / 9 / 7;
        let x = sudoku_i % self.n;
        let y = sudoku_i / self.n;
        (x, y)
    }

    pub fn sudokus_at_index(&self, i: usize) -> (Coords, Option<Coords>) {
        let block_i = i / 9;
        let block = match block_i % 7 {
            0 => TOP_CENTER_BLOCK,
            n => n + 2, // add to because top left and top right are skipped
        };
        let sudoku_i = block_i / 7;
        let x = sudoku_i % self.n;
        let y = sudoku_i / self.n;
        let other_coords = match block {
            TOP_LEFT_BLOCK => Some((x, (y + 1) % self.m)),
            TOP_RIGHT_BLOCK => Some(((x + 1) % self.n, y)),
            BOTTOM_LEFT_BLOCK => Some(((x + self.n - 1) % self.n, y)),
            BOTTOM_RIGHT_BLOCK => Some((x, (y + self.m - 1) % self.m)),
            _ => None,
        };
        ((x, y), other_coords)
    }

    /// Solve a sudoku with depth-first search
    /// (only changes cells containing zeros)
    ///
    /// Returns Err if no solution is found
    pub fn depth_first_solve(&mut self, sudoku_coords: Coords) -> Result<u64, NoSolution> {
        let mut guesses = Vec::new();
        let mut i = 0;
        let mut ignore_non_zero = true;
        let mut backtracks: u64 = 0;

        let indexes = self
            .sudoku(sudoku_coords)
            .indexes()
            .filter(|i| self.cells[*i] == 0)
            .collect::<Box<[_]>>();

        while i < indexes.len() {
            if ignore_non_zero && self.cells[indexes[i]] > 0 {
                // already filled, ignore this cell
                i += 1;
                continue;
            }

            // we will guess a value for this index
            self.cells[indexes[i]] += 1;
            while self.cells[indexes[i]] < 9 && self.cell_is_problematic(sudoku_coords, indexes[i])
            {
                self.cells[indexes[i]] += 1
            }

            if self.cell_is_problematic(sudoku_coords, indexes[i]) {
                // there is no solution, we should backtrack
                backtracks += 1;
                while self.cells[indexes[i]] == 9 {
                    self.cells[indexes[i]] = 0;
                    i = guesses.pop().ok_or(NoSolution)?;
                    ignore_non_zero = false;
                }
            } else {
                // let's try and see if this works
                guesses.push(i);
                i += 1;
                ignore_non_zero = true;
            }
        }

        Ok(backtracks)
    }

    pub fn solve_trivial_regions(&mut self, sudoku_coords: Coords) -> bool {
        let sudoku = self.sudoku(sudoku_coords);
        let mut unsolved_regions = Vec::<Vec<usize>>::new();
        for i in 0..9 {
            unsolved_regions.push(self.row(sudoku, i).indexes().collect());
            unsolved_regions.push(self.column(sudoku, i).indexes().collect());
            unsolved_regions.push(self.block(sudoku, i).indexes().collect());
        }

        let mut has_changed = true;
        let mut has_changed_at_all = false;
        while has_changed {
            has_changed = false;

            unsolved_regions.retain(|indexes| {
                let mut seen = Seen::new();
                let mut empties = 0;
                for i in indexes {
                    let value = self.cells[*i];
                    seen.add(value);
                    if value == 0 {
                        empties += 1;
                    }
                }

                if empties == 1 {
                    // get last missing value in region
                    let mut missing = 0;
                    for v in 1..=9 {
                        if !seen.contains(v) {
                            missing = v;
                            break;
                        }
                    }
                    // and put it in the empty spot
                    for i in indexes {
                        if self.cells[*i] == 0 {
                            self.cells[*i] = missing;
                            has_changed = true;
                            has_changed_at_all = true;
                            return false; // remove region because it is done
                        }
                    }
                }

                empties > 0 // keep regions with empty spots
            });
        }
        has_changed_at_all
    }
}

impl std::fmt::Debug for SudokuGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for x in 0..self.n {
            for y in 0..self.m {
                let s = self.sudoku((x, y));
                writeln!(f, "Sudoku ({x}, {y}):")?;
                writeln!(f, "┌───────┬───────┬───────┐")?;
                for i in 0..9 {
                    write!(f, "│ ")?;
                    for (j, v) in self.row(s, i).values().enumerate() {
                        let spaces = if j == 8 {
                            " │"
                        } else if j % 3 == 2 {
                            " │ "
                        } else {
                            " "
                        };
                        write!(f, "{}{}", v, spaces)?;
                    }
                    if i % 3 == 2 && i < 8 {
                        writeln!(f, "\n├───────┼───────┼───────┤")?;
                    } else {
                        writeln!(f)?;
                    }
                }
                writeln!(f, "└───────┴───────┴───────┘")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_overlap_1x1() {
        let sg = SudokuGrid::new(1, 1);
        assert_eq!(
            sg.sudoku((0, 0)).block_start[TOP_LEFT_BLOCK],
            sg.sudoku((0, 0)).block_start[BOTTOM_RIGHT_BLOCK]
        );
        assert_eq!(
            sg.sudoku((0, 0)).block_start[TOP_RIGHT_BLOCK],
            sg.sudoku((0, 0)).block_start[BOTTOM_LEFT_BLOCK]
        );
    }

    #[test]
    fn grid_overlap_2x2() {
        let sg = SudokuGrid::new(2, 2);
        assert_eq!(
            sg.sudoku((0, 0)).block_start[TOP_LEFT_BLOCK],
            sg.sudoku((0, 1)).block_start[BOTTOM_RIGHT_BLOCK]
        );
        assert_eq!(
            sg.sudoku((0, 0)).block_start[TOP_RIGHT_BLOCK],
            sg.sudoku((1, 0)).block_start[BOTTOM_LEFT_BLOCK]
        );
        assert_eq!(
            sg.sudoku((0, 0)).block_start[BOTTOM_LEFT_BLOCK],
            sg.sudoku((1, 0)).block_start[TOP_RIGHT_BLOCK]
        );
        assert_eq!(
            sg.sudoku((0, 0)).block_start[BOTTOM_RIGHT_BLOCK],
            sg.sudoku((0, 1)).block_start[TOP_LEFT_BLOCK]
        );
    }

    #[test]
    fn grid_overlap_3x3() {
        let sg = SudokuGrid::new(3, 3);
        assert_eq!(
            sg.sudoku((0, 0)).block_start[TOP_LEFT_BLOCK],
            sg.sudoku((0, 1)).block_start[BOTTOM_RIGHT_BLOCK]
        );
        assert_eq!(
            sg.sudoku((0, 0)).block_start[TOP_RIGHT_BLOCK],
            sg.sudoku((1, 0)).block_start[BOTTOM_LEFT_BLOCK]
        );
        assert_eq!(
            sg.sudoku((0, 0)).block_start[BOTTOM_LEFT_BLOCK],
            sg.sudoku((2, 0)).block_start[TOP_RIGHT_BLOCK]
        );
        assert_eq!(
            sg.sudoku((0, 0)).block_start[BOTTOM_RIGHT_BLOCK],
            sg.sudoku((0, 2)).block_start[TOP_LEFT_BLOCK]
        );
    }

    #[test]
    fn get_block_row_column_values() {
        let mut sg = SudokuGrid::new(1, 1);
        sg.cells = vec![
            11, 12, 13, 14, 15, 16, 17, 18, 19, // top center block
            21, 22, 23, 24, 25, 26, 27, 28, 29, // middle left block
            31, 32, 33, 34, 35, 36, 37, 38, 39, // middle center block
            41, 42, 43, 44, 45, 46, 47, 48, 49, // middle right block
            51, 52, 53, 54, 55, 56, 57, 58, 59, // bottom left block
            61, 62, 63, 64, 65, 66, 67, 68, 69, // bottom center block
            71, 72, 73, 74, 75, 76, 77, 78, 79, // bottom right block
        ]
        .into();
        let s = sg.sudoku((0, 0));

        // blocks
        assert_eq!(
            sg.block(&s, TOP_LEFT_BLOCK)
                .values()
                .copied()
                .collect::<Vec<_>>(),
            [71, 72, 73, 74, 75, 76, 77, 78, 79]
        );
        assert_eq!(
            sg.block(&s, TOP_CENTER_BLOCK)
                .values()
                .copied()
                .collect::<Vec<_>>(),
            [11, 12, 13, 14, 15, 16, 17, 18, 19]
        );
        assert_eq!(
            sg.block(&s, TOP_RIGHT_BLOCK)
                .values()
                .copied()
                .collect::<Vec<_>>(),
            [51, 52, 53, 54, 55, 56, 57, 58, 59]
        );
        assert_eq!(
            sg.block(&s, MIDDLE_LEFT_BLOCK)
                .values()
                .copied()
                .collect::<Vec<_>>(),
            [21, 22, 23, 24, 25, 26, 27, 28, 29]
        );
        assert_eq!(
            sg.block(&s, MIDDLE_CENTER_BLOCK)
                .values()
                .copied()
                .collect::<Vec<_>>(),
            [31, 32, 33, 34, 35, 36, 37, 38, 39]
        );
        assert_eq!(
            sg.block(&s, MIDDLE_RIGHT_BLOCK)
                .values()
                .copied()
                .collect::<Vec<_>>(),
            [41, 42, 43, 44, 45, 46, 47, 48, 49]
        );
        assert_eq!(
            sg.block(&s, BOTTOM_LEFT_BLOCK)
                .values()
                .copied()
                .collect::<Vec<_>>(),
            [51, 52, 53, 54, 55, 56, 57, 58, 59]
        );
        assert_eq!(
            sg.block(&s, BOTTOM_CENTER_BLOCK)
                .values()
                .copied()
                .collect::<Vec<_>>(),
            [61, 62, 63, 64, 65, 66, 67, 68, 69]
        );
        assert_eq!(
            sg.block(&s, BOTTOM_RIGHT_BLOCK)
                .values()
                .copied()
                .collect::<Vec<_>>(),
            [71, 72, 73, 74, 75, 76, 77, 78, 79]
        );

        // rows
        assert_eq!(
            sg.row(&s, 0).values().copied().collect::<Vec<_>>(),
            [71, 72, 73, 11, 12, 13, 51, 52, 53]
        );
        assert_eq!(
            sg.row(&s, 1).values().copied().collect::<Vec<_>>(),
            [74, 75, 76, 14, 15, 16, 54, 55, 56]
        );
        assert_eq!(
            sg.row(&s, 2).values().copied().collect::<Vec<_>>(),
            [77, 78, 79, 17, 18, 19, 57, 58, 59]
        );
        assert_eq!(
            sg.row(&s, 3).values().copied().collect::<Vec<_>>(),
            [21, 22, 23, 31, 32, 33, 41, 42, 43]
        );
        assert_eq!(
            sg.row(&s, 4).values().copied().collect::<Vec<_>>(),
            [24, 25, 26, 34, 35, 36, 44, 45, 46]
        );
        assert_eq!(
            sg.row(&s, 5).values().copied().collect::<Vec<_>>(),
            [27, 28, 29, 37, 38, 39, 47, 48, 49]
        );
        assert_eq!(
            sg.row(&s, 6).values().copied().collect::<Vec<_>>(),
            [51, 52, 53, 61, 62, 63, 71, 72, 73]
        );
        assert_eq!(
            sg.row(&s, 7).values().copied().collect::<Vec<_>>(),
            [54, 55, 56, 64, 65, 66, 74, 75, 76]
        );
        assert_eq!(
            sg.row(&s, 8).values().copied().collect::<Vec<_>>(),
            [57, 58, 59, 67, 68, 69, 77, 78, 79]
        );

        // columns
        assert_eq!(
            sg.column(&s, 0).values().copied().collect::<Vec<_>>(),
            [71, 74, 77, 21, 24, 27, 51, 54, 57]
        );
        assert_eq!(
            sg.column(&s, 1).values().copied().collect::<Vec<_>>(),
            [72, 75, 78, 22, 25, 28, 52, 55, 58]
        );
        assert_eq!(
            sg.column(&s, 2).values().copied().collect::<Vec<_>>(),
            [73, 76, 79, 23, 26, 29, 53, 56, 59]
        );
        assert_eq!(
            sg.column(&s, 3).values().copied().collect::<Vec<_>>(),
            [11, 14, 17, 31, 34, 37, 61, 64, 67]
        );
        assert_eq!(
            sg.column(&s, 4).values().copied().collect::<Vec<_>>(),
            [12, 15, 18, 32, 35, 38, 62, 65, 68]
        );
        assert_eq!(
            sg.column(&s, 5).values().copied().collect::<Vec<_>>(),
            [13, 16, 19, 33, 36, 39, 63, 66, 69]
        );
        assert_eq!(
            sg.column(&s, 6).values().copied().collect::<Vec<_>>(),
            [51, 54, 57, 41, 44, 47, 71, 74, 77]
        );
        assert_eq!(
            sg.column(&s, 7).values().copied().collect::<Vec<_>>(),
            [52, 55, 58, 42, 45, 48, 72, 75, 78]
        );
        assert_eq!(
            sg.column(&s, 8).values().copied().collect::<Vec<_>>(),
            [53, 56, 59, 43, 46, 49, 73, 76, 79]
        );
    }

    #[test]
    fn get_sudoku_from_index() {
        let sg = SudokuGrid::new(3, 3);

        for x in 0..3 {
            for y in 0..3 {
                let start = sg.sudoku((x, y)).block_start[TOP_CENTER_BLOCK];
                for i in 0..9 * 7 {
                    assert_eq!(sg.sudoku_at_index(start + i), (x, y));
                    assert_eq!(sg.sudokus_at_index(start + i).0, (x, y));
                    let _ = sg.block_index_for(sg.sudoku((x, y)), start + i); // shouldn't panic
                }

                let start = sg.sudoku((x, y)).block_start[TOP_LEFT_BLOCK];
                assert_eq!(
                    start,
                    sg.sudoku((x, (y + 1) % 3)).block_start[BOTTOM_RIGHT_BLOCK]
                );
                for i in 0..9 {
                    assert_eq!(sg.sudokus_at_index(start + i).1, Some((x, y)));
                    let _ = sg.block_index_for(sg.sudoku((x, y)), start + i); // shouldn't panic
                }

                let start = sg.sudoku((x, y)).block_start[TOP_RIGHT_BLOCK];
                for i in 0..9 {
                    assert_eq!(sg.sudokus_at_index(start + i).1, Some((x, y)));
                    let _ = sg.block_index_for(sg.sudoku((x, y)), start + i); // shouldn't panic
                }
            }
        }
    }

    #[test]
    fn validate_region() {
        let mut sg = SudokuGrid::new(1, 1);
        sg.cells = (0..7 * 9).collect(); // 0..63

        // full validation
        assert!(Region(&sg, [1, 2, 3, 4, 5, 6, 7, 8, 9].into_iter()).validate(false));
        assert!(Region(&sg, [9, 6, 3, 8, 5, 2, 7, 4, 1].into_iter()).validate(false));
        assert!(!Region(&sg, [1, 2, 3, 4, 5, 6, 7, 0, 9].into_iter()).validate(false));
        assert!(!Region(&sg, [1, 2, 3, 4, 5, 6, 7, 10, 9].into_iter()).validate(false));
        assert!(!Region(&sg, [1, 2, 2, 4, 5, 6, 7, 8, 9].into_iter()).validate(false));
        assert!(!Region(&sg, [1, 2, 3, 4, 5, 6, 7, 9, 9].into_iter()).validate(false));
        assert!(!Region(&sg, [1, 2, 3, 4, 5, 6, 7, 8, 1].into_iter()).validate(false));

        // partial validation
        assert!(Region(&sg, [1, 2, 3, 4, 5, 6, 7, 8, 9].into_iter()).validate(true));
        assert!(Region(&sg, [9, 6, 3, 8, 5, 2, 7, 4, 1].into_iter()).validate(true));
        assert!(Region(&sg, [0, 0, 0, 0, 0, 0, 0, 0, 0].into_iter()).validate(true));
        assert!(Region(&sg, [0, 1, 0, 0, 0, 0, 6, 0, 0].into_iter()).validate(true));
        assert!(Region(&sg, [1, 2, 3, 4, 5, 6, 7, 0, 9].into_iter()).validate(true));
        assert!(!Region(&sg, [1, 2, 3, 4, 5, 6, 7, 10, 9].into_iter()).validate(true));
        assert!(!Region(&sg, [1, 2, 2, 4, 5, 6, 7, 8, 9].into_iter()).validate(true));
        assert!(!Region(&sg, [1, 2, 3, 4, 5, 6, 7, 9, 9].into_iter()).validate(true));
        assert!(!Region(&sg, [1, 2, 3, 4, 5, 6, 7, 8, 1].into_iter()).validate(true));
    }

    #[test]
    fn sudoku_indexes() {
        let sg = SudokuGrid::new(1, 1);
        let s = sg.sudoku((0, 0));
        let v = s.indexes().collect::<Vec<_>>();
        assert_eq!(v.len(), 9 * 9);
    }

    #[test]
    fn dfs_block_next_solution() {
        let mut sg = SudokuGrid::new(1, 1);
        let s = sg.sudoku((0, 0)).clone();
        let order = [4, 1, 7, 9, 2, 6, 5, 3, 8];
        let mut dfs_block = DfsBlock::new(&sg, (0, 0), TOP_LEFT_BLOCK, &order);

        assert!(dfs_block
            .next_solution(&mut sg)
            .is_ok_and(|backtracks| backtracks == 0));
        println!("{sg:?}");
        assert!(sg.block(&s, TOP_LEFT_BLOCK).values().eq(&order));

        assert!(dfs_block
            .next_solution(&mut sg)
            .is_ok_and(|backtracks| backtracks == 1));
        println!("{sg:?}");
        assert!(sg
            .block(&s, TOP_LEFT_BLOCK)
            .values()
            .eq(&[4, 1, 7, 9, 2, 6, 5, 8, 3]));

        assert!(dfs_block.next_solution(&mut sg).is_ok_and(|backtracks| {
            println!("backtracks: {backtracks}");
            backtracks == 2
        }));
        println!("{sg:?}");
        assert!(sg
            .block(&s, TOP_LEFT_BLOCK)
            .values()
            .eq(&[4, 1, 7, 9, 2, 6, 3, 5, 8]));
    }
}
