import init, { generate, get_cell_index } from "../pkg/infinite_sudoku.js";
import glSetup from "./webgl.js";

// Sudoku grid size
const [n, m] = [3, 3];

const canvas = document.getElementsByTagName("canvas")[0];
const [_, gl] = await Promise.all([init(), glSetup(canvas)]);

const pixel_ratio = (window.devicePixelRatio);
const u_window_resolution = gl.uniform("u_window_resolution", "2fv",
    [window.innerWidth * pixel_ratio, window.innerHeight * pixel_ratio]);

const u_mouse_coords = gl.uniform("u_mouse_coords", "2fv", [0, 0]);
const u_selected_cell = gl.uniform("u_selected_cell", "2fv", [Infinity, Infinity]);

const u_number_texture = gl.texture("u_numbers_texture");
u_number_texture.loadImage("./assets/numbers1024.png", 0)
    .then(() => u_number_texture.activateMipmap());

const u_sudoku = gl.texture("u_sudoku", gl.internal.NEAREST);

// Generate sudoku
let data = generate(n, m);
console.log(data);

function updateSudokuData() {
    u_sudoku.setSourceArray(data, 7 * 9, n * m, gl.internal.LUMINANCE);
}
updateSudokuData();

gl.uniform("u_world_size", "2fv", [n, m]);

let inv_scale_factor = 1;
let inv_scale = 2 ** inv_scale_factor * 3 / 256;
const u_translate = gl.uniform("u_translate", "2fv", [0.0, 0.0]);
const u_inv_scale = gl.uniform("u_inv_scale", "1f", inv_scale);

/**
 * @param {number} zoom_delta 
 * @param {number} client_x 
 * @param {number} client_y
 */
function zoomInTo(zoom_delta, client_x, client_y) {
    if (zoom_delta == 0) return;

    inv_scale_factor -= zoom_delta;
    if (inv_scale_factor < -2) {
        inv_scale_factor = -2;
        return;
    }
    if (inv_scale_factor > 7) {
        inv_scale_factor = 7;
        return;
    }

    const old_inv_scale = inv_scale;
    inv_scale = 2 ** inv_scale_factor * 3 / 256;
    u_inv_scale.set(inv_scale);
    const d_inv_scale = (inv_scale - old_inv_scale) / old_inv_scale;

    const dx = (client_x - 0.5 * window.innerWidth) * old_inv_scale;
    const dy = (client_y - 0.5 * window.innerHeight) * old_inv_scale;
    u_translate.setf(([x, y]) => [x - d_inv_scale * dx, y + d_inv_scale * dy]);
}

document.addEventListener("wheel", (ev) => {
    const zoom_delta = ev.deltaY > 0 ? -1 : ev.deltaY < 0 ? 1 : 0;
    zoomInTo(0.25 * zoom_delta, ev.clientX, ev.clientY);
});

/**
 * @type {Object.<string, (ev: KeyboardEvent) => void>}
 */
const key_handlers = {
    "ArrowLeft": () => { u_translate.setf(([x, y]) => [x - 1, y]); u_selected_cell.setf(([x, y]) => [x - 1, y]); },
    "ArrowRight": () => { u_translate.setf(([x, y]) => [x + 1, y]); u_selected_cell.setf(([x, y]) => [x + 1, y]); },
    "ArrowUp": () => { u_translate.setf(([x, y]) => [x, y + 1]); u_selected_cell.setf(([x, y]) => [x, y - 1]); },
    "ArrowDown": () => { u_translate.setf(([x, y]) => [x, y - 1]); u_selected_cell.setf(([x, y]) => [x, y + 1]); },
    "-": () => zoomInTo(-1, mx, my),
    "+": () => zoomInTo(1, mx, my),
    "=": () => zoomInTo(1, mx, my),
    "1": () => setSelectedCell(1),
    "2": () => setSelectedCell(2),
    "3": () => setSelectedCell(3),
    "4": () => setSelectedCell(4),
    "5": () => setSelectedCell(5),
    "6": () => setSelectedCell(6),
    "7": () => setSelectedCell(7),
    "8": () => setSelectedCell(8),
    "9": () => setSelectedCell(9),
    "Backspace": () => setSelectedCell(0),
    "Delete": () => setSelectedCell(0),
    "Escape": () => u_selected_cell.set([Infinity, Infinity]),
}
document.addEventListener("keydown", (ev) => key_handlers[ev.key]?.(ev));

let clicked = false;
let dragging = false;
let [mx, my] = [0, 0];
canvas.addEventListener("mousedown", (ev) => {
    if (ev.button == 0 || ev.button == 1) {
        clicked = true;
    }
});

document.addEventListener("mousemove", (ev) => {
    const x = ev.clientX * pixel_ratio;
    const y = ev.clientY * pixel_ratio;

    if (ev.buttons == 0) { // no mouse buttons pressed
        clicked = false;
        dragging = false;
    }

    if (clicked) {
        dragging = true;
        // Click & drag to pan view
        let dx = x - mx;
        let dy = y - my;
        u_translate.setf(([x, y]) => [x - dx * inv_scale, y + dy * inv_scale]);
    }

    [mx, my] = [x, y];
});


document.addEventListener("mouseup", (ev) => {
    if (dragging) {
        clicked = false;
        dragging = false;
        return;
    }

    if (ev.button == 0) {
        // set selected cell
        // matches computation of u_mouse_coords
        // TODO: make also work for touchscreens
        let [tx, ty] = u_translate.get();
        let bx = (ev.clientX * pixel_ratio - 0.5 * window.innerWidth) * pixel_ratio * inv_scale + tx;
        let by = (ev.clientY * pixel_ratio - 0.5 * window.innerHeight) * pixel_ratio * inv_scale - ty + 1;

        u_selected_cell.set([bx, by]);
    }
});

/**
 * Set the value of the selected cell
 * @param {number} num 
 */
function setSelectedCell(num) {
    const [bx, by] = u_selected_cell.get();
    if (!isFinite(bx) || !isFinite(by)) {
        return;
    }

    let fb4x = Math.floor((((bx % 12) + 12) % 12) / 3);
    let fb4y = Math.floor((((by % 12) + 12) % 12) / 3);

    if ((fb4x == 1 && fb4y == 0) || (fb4x == 3 && fb4y == 2)) {
        return; // gray block
    }

    const top_left = fb4x == 0 && fb4y < 2;
    const top_right = fb4x >= 2 && fb4y < 2;
    const bottom_right = fb4x == 3 && fb4y == 3;

    // get sudoku coord
    let fx = Math.floor(bx / 12);
    let fy = Math.floor(by / 12);
    let sx = fx - fy;
    let sy = -fx - fy;
    sy += +top_left;
    sx += +top_right;
    sy += -bottom_right;
    sx = ((Math.floor(sx + 0.5) % n) + n) % n;
    sy = ((Math.floor(sy + 0.5) % m) + m) % m;

    // get sudoku cell index (based on uv)
    let scx = ((Math.floor(bx + 6 * +top_left - 6 * +top_right - 6 * +bottom_right) % 12) + 12) % 12;
    let scy = ((Math.floor(by - 3 + 6 * +top_left + 6 * +top_right - 6 * +bottom_right) % 12) + 12) % 12;

    let i = get_cell_index(n, m, sx, sy, scx, scy);
    if ((data[i] & 16) != 16 && data[i] != 0) {
        console.log("cannot edit constant value", data[i]);
        return;
    }

    data[i] = (num & 15) + 16; // user-specified flag

    updateSudokuData();

}

/**
 * Get center of touches
 * @param {TouchList} touches
 * @return {[number, number]} center
 */
function computeTouchCenter(touches) {
    let cx = 0;
    let cy = 0;
    for (let touch of touches) {
        cx += touch.clientX;
        cy += touch.clientY;
    }
    cx *= pixel_ratio / touches.length;
    cy *= pixel_ratio / touches.length;
    return [cx, cy]
}

let td = 0; // distance between touches (for pinch zoom)
canvas.addEventListener("touchstart", (ev) => {
    ev.preventDefault();

    [mx, my] = computeTouchCenter(ev.touches);

    if (ev.touches.length == 2) {
        let t1 = ev.touches[0];
        let t2 = ev.touches[1];
        td = Math.sqrt((t1.clientX - t2.clientX) ** 2 + (t1.clientY - t2.clientY) ** 2);
    }
});

document.addEventListener("touchmove", (ev) => {
    ev.preventDefault();

    let [cx, cy] = computeTouchCenter(ev.touches);
    let dx = cx - mx;
    let dy = cy - my;
    u_translate.setf(([x, y]) => [x - dx * inv_scale, y + dy * inv_scale]);

    [mx, my] = [cx, cy];

    if (ev.touches.length > 1) {
        let t1 = ev.touches[0];
        let t2 = ev.touches[1];
        let d = Math.sqrt((t1.clientX - t2.clientX) ** 2 + (t1.clientY - t2.clientY) ** 2);
        zoomInTo((d - td) / td, cx / pixel_ratio, cy / pixel_ratio);
        td = d;
    }
});

document.addEventListener("touchend", (ev) => {
    ev.preventDefault();

    [mx, my] = computeTouchCenter(ev.touches);
});

// Resize canvas
function resize() {
    canvas.width = Math.ceil(pixel_ratio * window.innerWidth);
    canvas.height = Math.ceil(pixel_ratio * window.innerHeight);
    u_window_resolution.set([
        window.innerWidth * pixel_ratio, window.innerHeight * pixel_ratio
    ]);
    gl.resize();
}
window.addEventListener("resize", resize);
resize();

// WebGL draw loop
function tick() {
    // map mouse screen coordinates to cell coordinates
    let [tx, ty] = u_translate.get();
    let bx = (mx - 0.5 * window.innerWidth * pixel_ratio) * inv_scale + tx;
    let by = (my - 0.5 * window.innerHeight * pixel_ratio) * inv_scale - ty + 1;
    u_mouse_coords.set([bx, by]);

    gl.draw();

    requestAnimationFrame(tick);
}
requestAnimationFrame(tick);
