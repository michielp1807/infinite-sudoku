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

/// Get randomly shuffled 1..=9
pub fn random_sequence() -> impl Iterator<Item = u8> {
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
pub fn generate() -> Box<[u8]> {
    console_error_panic_hook::set_once();

    let mut s = Sudoku::new_empty();

    // randomly fill first block
    for (i, v) in s.block(0).indexes().zip(random_sequence()) {
        s[i] = v;
    }

    // set opposite block to same numbers
    for (i, j) in s.block(8).indexes().zip(s.block(0).indexes()) {
        s[i] = s[j];
    }

    // randomly fill second block while satisfying constraints
    let values = random_sequence().collect::<Vec<u8>>();
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

    // set opposite block to same numbers (we can assume constraints hold here)
    for (i, j) in s.block(6).indexes().zip(s.block(2).indexes()) {
        s[i] = s[j];
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
