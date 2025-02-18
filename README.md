# Infinite Sudoku
An infinite sudoku, solvable in a finite amount of time!

Play at: https://michielp1807.github.io/infinite-sudoku

Build wasm binary with:
```
wasm-pack build --target web --no-pack && rm pkg/.gitignore
```

This project was inspired by the YouTube video ["I Created The World's Biggest Sudoku (with Code)"](https://youtu.be/0roAZFaqSjw) by Green Code. The Sudoku generation is based on the algorithm described in the paper ["Sudoku Puzzles Generating: from Easy to Evil"](https://zhangroup.aporc.org/images/files/Paper_3485.pdf).
