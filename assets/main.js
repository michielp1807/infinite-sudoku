import init, { generate, get_cell_index, mark_errors } from "../pkg/infinite_sudoku.js";
import glSetup from "./webgl.js";

const canvas = document.getElementsByTagName("canvas")[0];
const [_, gl] = await Promise.all([init(), glSetup(canvas)]);

const pixel_ratio = window.devicePixelRatio;
const u_window_resolution = gl.uniform("u_window_resolution", "2fv", [
    window.innerWidth * pixel_ratio,
    window.innerHeight * pixel_ratio,
]);

const u_mouse_coords = gl.uniform("u_mouse_coords", "2fv", [0, 0]);
const u_selected_cell = gl.uniform("u_selected_cell", "2fv", [Infinity, Infinity]);
const u_selected_value = gl.uniform("u_selected_value", "1i", 0);
u_selected_cell.onchange(updateSelectedValue);

const u_number_texture = gl.texture("u_numbers_texture");
u_number_texture.loadImage("./assets/numbers1024.png", 0).then(() => u_number_texture.activateMipmap());

const u_sudoku = gl.texture("u_sudoku", gl.internal.NEAREST);

// Sudoku grid size
let [n, m] = [1, 1];
const u_world_size = gl.uniform("u_world_size", "2fv", [n, m]);

let data = generate(1, 1, false);
function updateSudokuData() {
    u_sudoku.setSourceArray(data, 7 * 9, n * m, gl.internal.LUMINANCE);
}
updateSudokuData();

let in_menu = true;
const start_button = /** @type {HTMLButtonElement} */ (document.getElementById("start"));
const continue_button = /** @type {HTMLButtonElement} */ (document.getElementById("continue"));
const menu_container = /** @type {HTMLElement} */ (document.getElementById("menu-container"));

function hideMenu() {
    in_menu = false;
    u_inv_scale.set(inv_scale); // reset zoom animation
    menu_container.style.display = "none";
}

start_button.onclick = () => {
    n = 3;
    m = 3;
    data = generate(n, m, true);

    updateSudokuData();
    u_world_size.set([n, m]);

    hideMenu();
};

continue_button.disabled = !localStorage.infinite_sudoku_state;
continue_button.onclick = () => {
    const save_data = JSON.parse(localStorage.infinite_sudoku_state);
    n = save_data.n;
    m = save_data.m;
    data = Uint8Array.from(atob(save_data.data), (c) => c.charCodeAt(0));

    if (data.length != 9 * 7 * n * m) {
        alert("Save data has been corrupted (wrong size)");
        return;
    }

    updateSudokuData();
    u_world_size.set([n, m]);

    hideMenu();
};

function saveToLocalStorage() {
    const save_data = { n, m, data: btoa(String.fromCharCode(...data)) };
    localStorage.infinite_sudoku_state = JSON.stringify(save_data);
}

let inv_scale_factor = 1;
let inv_scale = (2 ** inv_scale_factor * 3) / 256;
const u_translate = gl.uniform("u_translate", "2fv", [0.0, 0.0]);
const u_inv_scale = gl.uniform("u_inv_scale", "1f", inv_scale);

/**
 * Get the cell index from a set of coordinates
 * @param {number} x
 * @param {number} y
 */
function getCellIndexFromCoords(x, y) {
    if (!isFinite(x) || !isFinite(y)) {
        return null;
    }

    let fb4x = Math.floor((((x % 12) + 12) % 12) / 3);
    let fb4y = Math.floor((((y % 12) + 12) % 12) / 3);

    if ((fb4x == 1 && fb4y == 0) || (fb4x == 3 && fb4y == 2)) {
        return null; // gray block
    }

    const top_left = fb4x == 0 && fb4y < 2;
    const top_right = fb4x >= 2 && fb4y < 2;
    const bottom_right = fb4x == 3 && fb4y == 3;

    // get sudoku coord
    let fx = Math.floor(x / 12);
    let fy = Math.floor(y / 12);
    let sx = fx - fy;
    let sy = -fx - fy;
    sy += +top_left;
    sx += +top_right;
    sy += -bottom_right;
    sx = ((Math.floor(sx + 0.5) % n) + n) % n;
    sy = ((Math.floor(sy + 0.5) % m) + m) % m;

    // get sudoku cell index (based on uv)
    let scx = ((Math.floor(x + 6 * +top_left - 6 * +top_right - 6 * +bottom_right) % 12) + 12) % 12;
    let scy = ((Math.floor(y - 3 + 6 * +top_left + 6 * +top_right - 6 * +bottom_right) % 12) + 12) % 12;

    return get_cell_index(n, m, sx, sy, scx, scy);
}

/**
 * Set the u_selected_value to match the value of u_selected_cell
 */
function updateSelectedValue() {
    const [bx, by] = u_selected_cell.get();
    const i = getCellIndexFromCoords(bx, by);
    if (i === null) {
        u_selected_value.set(0);
        return;
    }
    u_selected_value.set(data[i] & 15);
}

/**
 * Set the value of the selected cell
 * @param {number} num
 */
function fillSelectedCell(num) {
    const [bx, by] = u_selected_cell.get();
    const i = getCellIndexFromCoords(bx, by);
    if (i === null) {
        return;
    }

    if ((data[i] & 16) != 16 && (data[i] & 15) != 0) {
        console.log("cannot edit constant value", data[i]);
        return;
    }

    data[i] = (num & 15) + 16; // user-specified flag

    data = mark_errors(data, n, m);
    updateSudokuData();
    updateSelectedValue();
    saveToLocalStorage();
}

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
    inv_scale = (2 ** inv_scale_factor * 3) / 256;
    u_inv_scale.set(inv_scale);
    const d_inv_scale = (inv_scale - old_inv_scale) / old_inv_scale;

    const dx = (client_x - 0.5 * window.innerWidth) * old_inv_scale;
    const dy = (client_y - 0.5 * window.innerHeight) * old_inv_scale;
    u_translate.setf(([x, y]) => [x - d_inv_scale * dx, y + d_inv_scale * dy]);
}

canvas.addEventListener("wheel", (ev) => {
    const zoom_delta = ev.deltaY > 0 ? -1 : ev.deltaY < 0 ? 1 : 0;
    zoomInTo(0.25 * zoom_delta, ev.clientX, ev.clientY);
});

/**
 * @type {Object.<string, (ev: KeyboardEvent) => void>}
 */
const key_handlers = {
    ArrowLeft: () => {
        u_translate.setf(([x, y]) => [x - 1, y]);
        u_selected_cell.setf(([x, y]) => [x - 1, y]);
    },
    ArrowRight: () => {
        u_translate.setf(([x, y]) => [x + 1, y]);
        u_selected_cell.setf(([x, y]) => [x + 1, y]);
    },
    ArrowUp: () => {
        u_translate.setf(([x, y]) => [x, y + 1]);
        u_selected_cell.setf(([x, y]) => [x, y - 1]);
    },
    ArrowDown: () => {
        u_translate.setf(([x, y]) => [x, y - 1]);
        u_selected_cell.setf(([x, y]) => [x, y + 1]);
    },
    "-": () => zoomInTo(-1, mx, my),
    "+": () => zoomInTo(1, mx, my),
    "=": () => zoomInTo(1, mx, my),
    1: () => fillSelectedCell(1),
    2: () => fillSelectedCell(2),
    3: () => fillSelectedCell(3),
    4: () => fillSelectedCell(4),
    5: () => fillSelectedCell(5),
    6: () => fillSelectedCell(6),
    7: () => fillSelectedCell(7),
    8: () => fillSelectedCell(8),
    9: () => fillSelectedCell(9),
    Backspace: () => fillSelectedCell(0),
    Delete: () => fillSelectedCell(0),
    Escape: () => {
        u_selected_cell.set([Infinity, Infinity]);
    },
};
document.addEventListener("keydown", (ev) => key_handlers[ev.key]?.(ev));

let clicked = false;
let dragging = false;
let [mx, my] = [0, 0];
canvas.addEventListener("mousedown", (ev) => {
    if (ev.button == 0 || ev.button == 1) {
        clicked = true;
    }
});

canvas.addEventListener("mousemove", (ev) => {
    const x = ev.clientX * pixel_ratio;
    const y = ev.clientY * pixel_ratio;

    if (ev.buttons == 0) {
        // no mouse buttons pressed
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

/**
 * Set the selected cell based on a (x, y) mouse coordinate
 *
 * Matches computation of `u_mouse_coords`
 *
 * @param {number} x
 * @param {number} y
 */
function selectCell(x, y) {
    const [tx, ty] = u_translate.get();
    let bx = (x - 0.5 * window.innerWidth) * pixel_ratio * inv_scale + tx;
    let by = (y - 0.5 * window.innerHeight) * pixel_ratio * inv_scale - ty + 1;

    u_selected_cell.set([bx, by]);
}

canvas.addEventListener("mouseup", (ev) => {
    if (dragging) {
        clicked = false;
        dragging = false;
        return;
    }

    if (ev.button == 0) {
        selectCell(ev.clientX, ev.clientY);
    }
});

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
    return [cx, cy];
}

let tap = false; // true when just a single touch is used and not moving
let td = 0; // distance between touches (for pinch zoom)
canvas.addEventListener("touchstart", (ev) => {
    ev.preventDefault();

    [mx, my] = computeTouchCenter(ev.touches);

    tap = ev.touches.length == 1;

    if (ev.touches.length == 2) {
        let t1 = ev.touches[0];
        let t2 = ev.touches[1];
        td = Math.sqrt((t1.clientX - t2.clientX) ** 2 + (t1.clientY - t2.clientY) ** 2);
    }
});

canvas.addEventListener("touchmove", (ev) => {
    ev.preventDefault();

    tap = false;

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

canvas.addEventListener("touchend", (ev) => {
    ev.preventDefault();

    [mx, my] = computeTouchCenter(ev.touches);

    if (ev.touches.length == 0 && tap) {
        const x = ev.changedTouches[0].clientX;
        const y = ev.changedTouches[0].clientY;
        selectCell(x, y);
    }
});

// Resize canvas
function resize() {
    canvas.width = Math.ceil(pixel_ratio * window.innerWidth);
    canvas.height = Math.ceil(pixel_ratio * window.innerHeight);
    u_window_resolution.set([window.innerWidth * pixel_ratio, window.innerHeight * pixel_ratio]);
    gl.resize();
}
window.addEventListener("resize", resize);
resize();

/**
 * WebGL draw loop
 * @type {FrameRequestCallback}
 */
function tick(time) {
    // map mouse screen coordinates to cell coordinates
    let [tx, ty] = u_translate.get();
    let bx = (mx - 0.5 * window.innerWidth * pixel_ratio) * inv_scale + tx;
    let by = (my - 0.5 * window.innerHeight * pixel_ratio) * inv_scale - ty + 1;
    u_mouse_coords.set([bx, by]);

    if (in_menu) {
        // zoom animation
        u_inv_scale.set(inv_scale * (2 - Math.cos(0.00005 * time)));
    }

    gl.draw();

    requestAnimationFrame(tick);
}
requestAnimationFrame(tick);
