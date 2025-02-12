#[derive(Clone)]
pub struct Sudoku(pub [u8; 9 * 9]);

impl From<Sudoku> for Box<[u8]> {
    fn from(val: Sudoku) -> Self {
        val.0.into()
    }
}

/// A row, column, or block, described by a list of its 9 indexes
pub struct Region<'a, T: Iterator<Item = usize>>(&'a Sudoku, T);

impl<'a, T: Iterator<Item = usize>> Region<'a, T> {
    pub fn values(self) -> impl Iterator<Item = &'a u8> {
        self.1.map(|i| &self.0[i])
    }

    pub fn indexes(self) -> impl Iterator<Item = usize> {
        self.1
    }

    /// Check if region contains the numbers 1 to 9 (assuming the sequence is 9 long)
    pub fn validate(self) -> bool {
        let mut seen: u16 = 0; // bitmap of seen values
        for i in self.1 {
            let v = self.0[i];
            if v == 0 || v > 9 || seen & (2 << v) > 0 {
                return false;
            }
            seen |= 2 << v;
        }
        true
    }

    /// Check if region contains the numbers 1 to 9 (assuming the sequence is 9 long)
    ///
    /// Ignores empty (0) cells
    pub fn partial_validate(self) -> bool {
        let mut seen: u16 = 0; // bitmap of seen values
        for i in self.1 {
            let v = self.0[i];
            if v == 0 {
                continue;
            }
            if v > 9 || seen & (2 << v) > 0 {
                return false;
            }
            seen |= 2 << v;
        }
        true
    }
}

impl std::ops::Index<usize> for Sudoku {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::IndexMut<usize> for Sudoku {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Sudoku {
    pub fn new_empty() -> Self {
        Sudoku([0; 9 * 9])
    }

    pub fn row(&self, i: usize) -> Region<impl Iterator<Item = usize>> {
        assert!(i < 9, "row index out of bounds");
        let start = i * 9;
        let end = start + 9;
        Region(self, start..end)
    }

    pub fn column(&self, i: usize) -> Region<impl Iterator<Item = usize>> {
        assert!(i < 9, "column index out of bounds");
        Region(self, (0..(9 * 9)).skip(i).step_by(9))
    }

    pub fn block(&self, i: usize) -> Region<impl Iterator<Item = usize>> {
        assert!(i < 9, "block index out of bounds");
        Region(
            self,
            [0, 1, 2, 9, 10, 11, 18, 19, 20]
                .iter()
                .map(move |v| v + i * 3 + i / 3 * 9 * 2),
        )
    }

    /// Check if sudoku is solved correctly
    pub fn is_solved(&self) -> bool {
        // row constraint
        if !(0..9).all(|i| self.row(i).validate()) {
            return false;
        }

        // column constraint
        if !(0..9).all(|i| self.column(i).validate()) {
            return false;
        }

        // block constraint
        if !(0..9).all(|i| self.block(i).validate()) {
            return false;
        }

        true
    }

    /// Check if the cell at index i is problematic
    pub fn cell_is_problematic(&self, i: usize) -> bool {
        let row = i / 9;
        if !self.row(row).partial_validate() {
            return true;
        }

        let column = i % 9;
        if !self.column(column).partial_validate() {
            return true;
        }

        let block = column / 3 + 3 * (row / 3);
        if !self.block(block).partial_validate() {
            return true;
        }

        false
    }

    /// Solve a sudoku with depth-first search
    /// (only changes cells containing zeros)
    ///
    /// Returns Err if no solution is found
    pub fn solve(&mut self) -> Result<(), ()> {
        let mut guesses = Vec::new();
        let mut i = 0;
        let mut ignore_non_zero = true;

        while i < 9 * 9 {
            if ignore_non_zero && self[i] > 0 {
                // already filled, ignore this cell
                i += 1;
                continue;
            }

            // we will guess a value for this index
            self[i] += 1;
            while self.cell_is_problematic(i) && self[i] < 9 {
                self[i] += 1
            }

            if self.cell_is_problematic(i) {
                // there is no solution, we should backtrack
                self[i] = 0;
                i = guesses.pop().ok_or(())?;
                ignore_non_zero = false;
            } else {
                // let's try and see if this works
                guesses.push(i);
                i += 1;
                ignore_non_zero = true;
            }
        }

        Ok(())
    }
}

impl std::fmt::Debug for Sudoku {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for i in 0..9 {
            writeln!(f, "{:?}", self.row(i).values().collect::<Vec<_>>())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_row_values() {
        let s = Sudoku([
            1, 2, 3, 4, 5, 6, 7, 8, 9, // 0
            2, 0, 0, 0, 0, 0, 0, 0, 0, //
            3, 0, 0, 0, 0, 0, 0, 0, 0, //
            4, 0, 0, 1, 1, 1, 0, 0, 0, // 3
            5, 0, 0, 1, 0, 1, 0, 2, 0, //
            6, 0, 0, 1, 1, 1, 0, 0, 0, //
            7, 0, 0, 0, 0, 0, 0, 0, 0, // 6
            8, 0, 0, 0, 3, 0, 0, 4, 0, //
            9, 0, 0, 0, 0, 0, 0, 0, 0, //
        ]);

        assert!(s
            .row(0)
            .indexes()
            .eq([0, 1, 2, 3, 4, 5, 6, 7, 8].into_iter()));

        assert!(s.row(0).values().eq([1, 2, 3, 4, 5, 6, 7, 8, 9].iter()));
        assert!(s.row(1).values().eq([2, 0, 0, 0, 0, 0, 0, 0, 0].iter()));
        assert!(s.row(2).values().eq([3, 0, 0, 0, 0, 0, 0, 0, 0].iter()));
        assert!(s.row(3).values().eq([4, 0, 0, 1, 1, 1, 0, 0, 0].iter()));
        assert!(s.row(4).values().eq([5, 0, 0, 1, 0, 1, 0, 2, 0].iter()));
        assert!(s.row(5).values().eq([6, 0, 0, 1, 1, 1, 0, 0, 0].iter()));
        assert!(s.row(6).values().eq([7, 0, 0, 0, 0, 0, 0, 0, 0].iter()));
        assert!(s.row(7).values().eq([8, 0, 0, 0, 3, 0, 0, 4, 0].iter()));
        assert!(s.row(8).values().eq([9, 0, 0, 0, 0, 0, 0, 0, 0].iter()));
    }

    #[test]
    #[should_panic]
    fn get_invalid_row() {
        let _ = Sudoku::new_empty().row(9);
    }

    #[test]
    fn get_column_values() {
        let s = Sudoku([
            1, 2, 3, 4, 5, 6, 7, 8, 9, // 0
            2, 0, 0, 0, 0, 0, 0, 0, 0, //
            3, 0, 0, 0, 0, 0, 0, 0, 0, //
            4, 0, 0, 1, 1, 1, 0, 0, 0, // 3
            5, 0, 0, 1, 0, 1, 0, 2, 0, //
            6, 0, 0, 1, 1, 1, 0, 0, 0, //
            7, 0, 0, 0, 0, 0, 0, 0, 0, // 6
            8, 0, 0, 0, 3, 0, 0, 4, 0, //
            9, 0, 0, 0, 0, 0, 0, 0, 0, //
        ]);

        assert!(s
            .column(0)
            .indexes()
            .eq([0, 9, 18, 27, 36, 45, 54, 63, 72].into_iter()));

        assert!(s.column(0).values().eq([1, 2, 3, 4, 5, 6, 7, 8, 9].iter()));
        assert!(s.column(1).values().eq([2, 0, 0, 0, 0, 0, 0, 0, 0].iter()));
        assert!(s.column(2).values().eq([3, 0, 0, 0, 0, 0, 0, 0, 0].iter()));
        assert!(s.column(3).values().eq([4, 0, 0, 1, 1, 1, 0, 0, 0].iter()));
        assert!(s.column(4).values().eq([5, 0, 0, 1, 0, 1, 0, 3, 0].iter()));
        assert!(s.column(5).values().eq([6, 0, 0, 1, 1, 1, 0, 0, 0].iter()));
        assert!(s.column(6).values().eq([7, 0, 0, 0, 0, 0, 0, 0, 0].iter()));
        assert!(s.column(7).values().eq([8, 0, 0, 0, 2, 0, 0, 4, 0].iter()));
        assert!(s.column(8).values().eq([9, 0, 0, 0, 0, 0, 0, 0, 0].iter()));
    }

    #[test]
    #[should_panic]
    fn get_invalid_column() {
        let _ = Sudoku::new_empty().column(9);
    }

    #[test]
    fn get_block_values() {
        let s = Sudoku([
            1, 2, 3, 4, 5, 6, 7, 8, 9, // 0
            2, 0, 0, 0, 0, 0, 0, 0, 0, //
            3, 0, 0, 0, 0, 0, 0, 0, 0, //
            4, 0, 0, 1, 1, 1, 0, 0, 0, // 3
            5, 0, 0, 1, 0, 1, 0, 2, 0, //
            6, 0, 0, 1, 1, 1, 0, 0, 0, //
            7, 0, 0, 0, 0, 0, 0, 0, 0, // 6
            8, 0, 0, 0, 3, 0, 0, 4, 0, //
            9, 0, 0, 0, 0, 0, 0, 0, 0, //
        ]);

        assert!(s
            .block(0)
            .indexes()
            .eq([0, 1, 2, 9, 10, 11, 18, 19, 20].into_iter()));

        assert!(s.block(0).values().eq([1, 2, 3, 2, 0, 0, 3, 0, 0].iter()));
        assert!(s.block(1).values().eq([4, 5, 6, 0, 0, 0, 0, 0, 0].iter()));
        assert!(s.block(2).values().eq([7, 8, 9, 0, 0, 0, 0, 0, 0].iter()));
        assert!(s.block(3).values().eq([4, 0, 0, 5, 0, 0, 6, 0, 0].iter()));
        assert!(s.block(4).values().eq([1, 1, 1, 1, 0, 1, 1, 1, 1].iter()));
        assert!(s.block(5).values().eq([0, 0, 0, 0, 2, 0, 0, 0, 0].iter()));
        assert!(s.block(6).values().eq([7, 0, 0, 8, 0, 0, 9, 0, 0].iter()));
        assert!(s.block(7).values().eq([0, 0, 0, 0, 3, 0, 0, 0, 0].iter()));
        assert!(s.block(8).values().eq([0, 0, 0, 0, 4, 0, 0, 0, 0].iter()));
    }

    #[test]
    #[should_panic]
    fn get_invalid_block() {
        let _ = Sudoku::new_empty().block(9);
    }

    #[test]
    fn validate_region() {
        let s = Sudoku(std::array::from_fn(|i| i as u8)); // 0..81
        assert!(Region(&s, [1, 2, 3, 4, 5, 6, 7, 8, 9].into_iter()).validate());
        assert!(Region(&s, [9, 6, 3, 8, 5, 2, 7, 4, 1].into_iter()).validate());
        assert!(!Region(&s, [1, 2, 3, 4, 5, 6, 7, 0, 9].into_iter()).validate());
        assert!(!Region(&s, [1, 2, 3, 4, 5, 6, 7, 10, 9].into_iter()).validate());
        assert!(!Region(&s, [1, 2, 2, 4, 5, 6, 7, 8, 9].into_iter()).validate());
        assert!(!Region(&s, [1, 2, 3, 4, 5, 6, 7, 9, 9].into_iter()).validate());
        assert!(!Region(&s, [1, 2, 3, 4, 5, 6, 7, 8, 1].into_iter()).validate());
    }

    #[test]
    fn is_solved() {
        let s = Sudoku::new_empty();
        assert!(!s.is_solved(), "all 0s should be invalid:\n{:?}", s);

        let s = Sudoku([
            1, 2, 3, 4, 5, 6, 7, 8, 9, // 0
            4, 5, 6, 7, 8, 9, 1, 2, 3, //
            7, 8, 9, 1, 2, 3, 4, 5, 6, //
            2, 3, 1, 5, 6, 4, 8, 9, 7, // 3
            5, 6, 4, 8, 9, 7, 2, 3, 1, //
            8, 9, 7, 2, 3, 1, 5, 6, 4, //
            3, 1, 2, 6, 4, 5, 9, 7, 8, // 6
            6, 4, 5, 9, 7, 8, 3, 1, 2, //
            9, 7, 8, 3, 1, 2, 6, 4, 5, //
        ]);
        assert!(s.is_solved(), "should be correct:\n{:?}", s);

        let mut s2 = s.clone();
        s2[0] = 2;
        s2[1] = 1;
        assert!(!s2.is_solved(), "wrong first row:\n{:?}", s2);

        let mut s2 = s.clone();
        s2[0] = 4;
        s2[9] = 1;
        assert!(!s2.is_solved(), "wrong first column:\n{:?}", s2);

        let mut s2 = s.clone();
        s2[37] = 0;
        assert!(!s2.is_solved(), "contains 0:\n{:?}", s2);

        let s3 = Sudoku([
            1, 2, 3, 4, 5, 6, 7, 8, 9, // 0
            2, 3, 1, 5, 6, 4, 8, 9, 7, //
            3, 1, 2, 6, 4, 5, 9, 7, 8, //
            4, 5, 6, 7, 8, 9, 1, 2, 3, // 3
            5, 6, 4, 8, 9, 7, 2, 3, 1, //
            6, 4, 5, 9, 7, 8, 3, 1, 2, //
            7, 8, 9, 1, 2, 3, 4, 5, 6, // 6
            8, 9, 7, 2, 3, 1, 5, 6, 4, //
            9, 7, 8, 3, 1, 2, 6, 4, 5, //
        ]);
        assert!(!s3.is_solved(), "wrong blocks:\n{:?}", s3);
    }

    #[test]
    fn partial_validate_region() {
        let s = Sudoku(std::array::from_fn(|i| i as u8)); // 0..81
        assert!(Region(&s, [1, 2, 3, 4, 5, 6, 7, 8, 9].into_iter()).partial_validate());
        assert!(Region(&s, [9, 6, 3, 8, 5, 2, 7, 4, 1].into_iter()).partial_validate());
        assert!(Region(&s, [0, 0, 0, 0, 0, 0, 0, 0, 0].into_iter()).partial_validate());
        assert!(Region(&s, [0, 1, 0, 0, 0, 0, 6, 0, 0].into_iter()).partial_validate());
        assert!(Region(&s, [1, 2, 3, 4, 5, 6, 7, 0, 9].into_iter()).partial_validate());
        assert!(!Region(&s, [1, 2, 3, 4, 5, 6, 7, 10, 9].into_iter()).partial_validate());
        assert!(!Region(&s, [1, 2, 2, 4, 5, 6, 7, 8, 9].into_iter()).partial_validate());
        assert!(!Region(&s, [1, 2, 3, 4, 5, 6, 7, 9, 9].into_iter()).partial_validate());
        assert!(!Region(&s, [1, 2, 3, 4, 5, 6, 7, 8, 1].into_iter()).partial_validate());
    }

    #[test]
    fn solve() {
        let mut s = Sudoku::new_empty();
        s.solve().unwrap();
    }
}
