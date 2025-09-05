mod sudokus;
use sudokus::{DfsBlock, SudokuGrid, BOTTOM_LEFT_BLOCK, BOTTOM_RIGHT_BLOCK, TOP_RIGHT_BLOCK};

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;
}

// Console log with format string
macro_rules! log {
    ($($arg:tt)*) => (log(&format!($($arg)*)))
}

pub fn random_int(max: usize) -> usize {
    ((max as f64) * random()) as usize
}

/// Get randomly shuffled 1..=9
pub fn generate_random_sequence() -> impl Iterator<Item = u8> {
    let mut values: Vec<u8> = (1..=9).collect();
    (0..9).map(move |_| {
        if values.len() > 1 {
            values.swap_remove(random_int(values.len()))
        } else {
            values[0]
        }
    })
}

/// Generate a solved sudoku based on random input data
#[wasm_bindgen]
pub fn generate(n: usize, m: usize) -> Box<[u8]> {
    console_error_panic_hook::set_once();

    let mut sg = SudokuGrid::new(n, m);

    // fill corners
    for x in 0..n {
        for y in 0..m {
            let sudoku = sg.sudoku(x, y);

            // randomized depth-first solve block
            let random_values = generate_random_sequence().collect::<Box<[u8]>>();
            let backtracks = DfsBlock::new(&sg, &sudoku, BOTTOM_LEFT_BLOCK, &random_values)
                .next_solution(&mut sg)
                .unwrap_or_else(|_| panic!("Could not solve\n{sg:?}"));
            log!("Sudoku ({x}, {y}) bottom left: {backtracks} backtracks");
        }

        for y in 0..m {
            let sudoku = sg.sudoku(x, y);

            // randomized depth-first solve block
            let random_values = generate_random_sequence().collect::<Box<[u8]>>();
            let mut backtracks = 0;
            let mut dfs = DfsBlock::new(&sg, &sudoku, BOTTOM_RIGHT_BLOCK, &random_values);
            if dfs.next_solution(&mut sg).is_err() {
                let mut other_dfs = DfsBlock::new(&sg, &sudoku, TOP_RIGHT_BLOCK, &random_values);
                other_dfs.reset(&mut sg);
                other_dfs
                    .next_solution(&mut sg)
                    .expect("Could not solve other_dfs");

                let other_sudoku = sg.sudoku(x, (y + sg.m - 1) % sg.m);
                let mut other_other_dfs =
                    DfsBlock::new(&sg, &other_sudoku, TOP_RIGHT_BLOCK, &random_values);
                other_other_dfs.reset(&mut sg);
                other_other_dfs
                    .next_solution(&mut sg)
                    .expect("Could not solve other_other_dfs");

                dfs.reset(&mut sg);
                // TODO: fix this - it seems to generate unsolvable sudokus about 50% of the time where the other_other_dfs is empty
                while dfs.next_solution(&mut sg).is_err() {
                    if other_dfs.next_solution(&mut sg).is_err() {
                        other_dfs.reset(&mut sg);
                        other_dfs
                            .next_solution(&mut sg)
                            .expect("Could not solve other_dfs");
                        other_other_dfs
                            .next_solution(&mut sg)
                            .expect("Could not solve!");
                    }
                    dfs.reset(&mut sg);
                    backtracks += 1;
                }
            }
            log!("Sudoku ({x}, {y}) bottom right: {backtracks} block backtracks");
        }
    }

    log!("{sg:?}");

    // solve sudokus
    // (I assume it is always possible to solve them with any valid corner blocks)
    let mut solve_total_backtracks = 0;
    for x in 0..n {
        for y in 0..m {
            let backtracks = sg
                .depth_first_solve(&sg.sudoku(x, y))
                .expect("sudoku should be solvable");
            log!("Sudoku ({x}, {y}) solve: {backtracks} backtracks");
            solve_total_backtracks += backtracks;
        }
    }

    log!("{sg:?}");
    log!("solve_total_backtracks: {solve_total_backtracks}");

    // punch holes

    // for x in 0..n {
    //     for y in 0..m {
    //         let sudoku = sg.sudoku(x, y);
    //         let indexes = sg
    //             .block(&sudoku, MIDDLE_CENTER_BLOCK)
    //             .indexes()
    //             .collect::<Vec<_>>();
    //         sg.cells[indexes[4]] = 0;
    //     }
    // }

    sg.sudoku_rows().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doable_solve0() {
        let mut sg = SudokuGrid::new(2, 2);
        let s = sg.sudoku(0, 0);
        sg.set_block(&s, sudokus::TOP_LEFT_BLOCK, 1..=9);

        println!("{:?}", sg);

        let backtracks = sg.depth_first_solve(&s).unwrap();
        println!("Backtracks: {backtracks}");
        assert!(sg.is_solved(&s));
    }

    #[test]
    fn doable_solve1() {
        let mut sg = SudokuGrid::new(2, 2);
        let s = sg.sudoku(0, 0);
        sg.set_block(&s, sudokus::MIDDLE_CENTER_BLOCK, 1..=9);

        println!("{:?}", sg);

        let backtracks = sg.depth_first_solve(&s).unwrap();
        println!("Backtracks: {backtracks}");
        assert!(sg.is_solved(&s));
    }

    #[test]
    fn doable_solve2() {
        let mut sg = SudokuGrid::new(2, 2);
        let s = sg.sudoku(0, 0);
        sg.set_block(&s, sudokus::BOTTOM_RIGHT_BLOCK, 1..=9);

        println!("{:?}", sg);

        let backtracks = sg.depth_first_solve(&s).unwrap();
        println!("Backtracks: {backtracks}");
        assert!(sg.is_solved(&s));
    }

    #[test]
    fn doable_solve3() {
        let mut sg = SudokuGrid::new(2, 2);
        let s = sg.sudoku(0, 0);
        sg.set_block(&s, sudokus::BOTTOM_RIGHT_BLOCK, 1..=7);

        println!("{:?}", sg);

        let backtracks = sg.depth_first_solve(&s).unwrap();
        println!("Backtracks: {backtracks}");
        assert!(sg.is_solved(&s));
    }

    #[test]
    fn difficult_solve() {
        let mut sg = SudokuGrid::new(2, 2);
        let s = sg.sudoku(0, 0);
        sg.set_block(&s, sudokus::BOTTOM_RIGHT_BLOCK, 1..=8);

        println!("{:?}", sg);

        // depth-first search will need to backtrack a lot because it only realizes at
        // the very end the last cell needs to be a 9
        // TODO: verify that this is why this happens and it is not actually stuck in an
        // infinite loop or something (it seems to backtrack more than 850,000,000 times
        // without using `solve_trivial_regions`)
        sg.solve_trivial_regions(&s); // this makes it a lot easier :)
        println!("{:?}", sg);
        let backtracks = sg.depth_first_solve(&s).unwrap();
        println!("{:?}", sg);

        println!("Backtracks: {backtracks}");
        assert!(sg.is_solved(&s));
    }
}
