mod sudoku;
use sudoku::Sudoku;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;
}

pub fn random_int(max: usize) -> usize {
    ((max as f64) * random()) as usize
}

/// Generate a solved sudoku based on random input data
#[wasm_bindgen]
pub fn generate() -> Box<[u8]> {
    console_error_panic_hook::set_once();

    let mut s = Sudoku::new_empty();

    // add some random numbers to randomize the sudoku
    // let mut i = 0;
    // while i < 36 {
    //     let r = random_int(9) + 1;
    //     s[i] = r;
    //     while s.cell_is_problematic(i) {
    //         s[i] = (s[i] % 9) + 1;
    //         if s[i] == r {
    //             log("Oops! This is impossible...");
    //             return generate();
    //         }
    //     }
    //     i += 1;
    // }

    // randomly fill first block
    let mut values: Vec<u8> = (1..=9).collect();
    for i in s.block(0).indexes() {
        s[i] = values.swap_remove(random_int(values.len()));
    }

    // set opposite block to same numbers
    let values: Vec<u8> = s.block(0).values().map(|v| *v).collect();
    for (i, index) in s.block(8).indexes().enumerate() {
        s[index] = values[i];
    }

    // randomly fill second block while satisfying constraints
    let mut values: Vec<u8> = (1..=9).collect();
    let values: Vec<u8> = (0..9)
        .map(|_| values.swap_remove(random_int(values.len())))
        .collect(); // randomly shuffled 1..=9
    let mut vi = [0usize; 9]; // value index (which random value is it using?)
    let indexes: Vec<usize> = s.block(2).indexes().collect();
    let mut i = 0;
    while i < 9 {
        let index = indexes[i];
        s[index] = values[vi[i]];

        while s.cell_is_problematic(index) && vi[i] < 8 {
            vi[i] += 1;
            s[index] = values[vi[i]];
        }

        if s.cell_is_problematic(index) {
            // there is no solution, we should backtrack
            while vi[i] == 8 {
                s[indexes[i]] = 0;
                vi[i] = 0;
                if i == 0 {
                    unreachable!("no solution");
                }
                i -= 1;
            }
            vi[i] += 1;
        } else {
            // let's try and see if this works
            i += 1;
        }
    }

    // set opposite block to same numbers (I think we can assume constraints to hold here?)
    let values: Vec<u8> = s.block(2).values().map(|v| *v).collect();
    for (i, index) in s.block(6).indexes().enumerate() {
        s[index] = values[i];
    }

    log(format!("{:?}", s).as_str());

    let res = s.solve();
    log(format!("Solve result: {:?}", res).as_str());

    log(format!("{:?}", s).as_str());
    log(format!("Is valid: {}", s.is_solved()).as_str());

    if res == Ok(()) {
        s.into()
    } else {
        log("Oops! This is unsolvable...");
        generate()
    }
}
