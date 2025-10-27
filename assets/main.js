import init, { generate, get_cell_index } from "../pkg/infinite_sudoku.js";
import glSetup from "./webgl.js";

// Sudoku grid size
const n = 3;
const m = 3;

const canvas = document.getElementsByTagName("canvas")[0];
const [_, gl] = await Promise.all([init(), glSetup(canvas)]);

const pixel_ratio = (window.devicePixelRatio);
const u_window_resolution = gl.uniform("u_window_resolution", "2fv",
    [window.innerWidth * pixel_ratio, window.innerHeight * pixel_ratio]);

const u_mouse_coords = gl.uniform("u_mouse_coords", "2fv", [0, 0]);

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

const u_world_size = gl.uniform("u_world_size", "2fv", [n, m]);
// TODO: change how the multiple sudokus are stored and handled


/** @type {[number, number]} */
let translate = [0.0, 0.0];
let inv_scale_factor = 1;
let inv_scale = 2 ** inv_scale_factor * 3 / 256;
const u_translate = gl.uniform("u_translate", "2fv", translate);
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

    const x = (client_x - 0.5 * window.innerWidth) * old_inv_scale;
    const y = (client_y - 0.5 * window.innerHeight) * old_inv_scale;
    translate[0] -= d_inv_scale * x;
    translate[1] += d_inv_scale * y;
    u_translate.set(translate);
}

document.addEventListener("wheel", (ev) => {
    const zoom_delta = ev.deltaY > 0 ? -1 : ev.deltaY < 0 ? 1 : 0;
    zoomInTo(0.25 * zoom_delta, ev.clientX, ev.clientY);
});

/**
 * @type {Object.<string, (ev: KeyboardEvent) => void>}
 */
const key_handlers = {
    "ArrowLeft": () => { translate[0] -= 0.333 * 128 * inv_scale; u_translate.set(translate); },
    "ArrowRight": () => { translate[0] += 0.333 * 128 * inv_scale; u_translate.set(translate); },
    "ArrowUp": () => { translate[1] += 0.333 * 128 * inv_scale; u_translate.set(translate); },
    "ArrowDown": () => { translate[1] -= 0.333 * 128 * inv_scale; u_translate.set(translate); },
    "-": () => zoomInTo(-1, mx, my),
    "+": () => zoomInTo(1, mx, my),
    "=": () => zoomInTo(1, mx, my),
}
document.addEventListener("keydown", (ev) => key_handlers[ev.key]?.(ev));

let clicked = false;
let dragging = false;
let [mx, my] = [0, 0];
canvas.addEventListener("mousedown", (ev) => {
    if (ev.button == 0 || ev.button == 1) {
        clicked = true;
    }

    if (ev.button == 0) {
        // matches computation of u_mouse_coords
        let bx = (ev.clientX * pixel_ratio - 0.5 * window.innerWidth) * pixel_ratio * inv_scale + translate[0];
        let by = (ev.clientY * pixel_ratio - 0.5 * window.innerHeight) * pixel_ratio * inv_scale - translate[1] + 1;

        // get sudoku coord
        let sx = ((Math.floor((bx - by + 6) / 12) % n) + n) % n;
        let sy = ((Math.floor((bx + by + 9) / 12) % m) + m) % m;

        // get sudoku cell index (based on uv)
        let offset = 6 * (1 - ((sx + sy) % 2));
        let scx = Math.floor((((bx + offset) % 12) + 12) % 12);
        let scy = Math.floor((((by + offset) % 12) + 12) % 12);

        console.log(`Clicked (${bx},${by}): sudoku (${sx},${sy}) - (${scx},${scy})`);
        // this still seems a bit broken for n > 2 / m > 2
        if (scx < 9 && scy < 9) {
            const value = 3 + 16; // +16 is the user_entered flag

            // TODO: update to use new data format
            // let i = get_cell_index(n, m, sx, sy, scx, scy);
            let i = (sx + sy * n) * 9 * 9 + scx + scy * 9;

            if ((data[i] & 16) != 16 && data[i] != 0) {
                console.log("cannot edit constant value", data[i]);
                return;
            }

            data[i] = value;

            // also set the other (not necessary in new data format)
            if (scx < 3 && scy < 3) {
                console.log("top left");
                sy = (sy - 1 + m) % m;
                scx += 6;
                scy += 6;
            } else if (scx < 3 && scy >= 6) {
                console.log("bottom left");
                sx = (sx + 1) % n;
                scx += 6;
                scy -= 6;
            } else if (scx >= 6 && scy >= 6) {
                console.log("bottom right");
                sy = (sy + 1) % m;
                scx -= 6;
                scy -= 6;
            } else if (scx >= 6 && scy < 3) {
                console.log("top right");
                sx = (sx - 1 + n) % n;
                scx -= 6;
                scy += 6;
            }
            console.log(`sudoku2 (${sx},${sy}) - (${scx},${scy})`);

            let j = (sx + sy * n) * 9 * 9 + scx + scy * 9;
            data[j] = value;

            console.log("i:", i, ", j:", j);
            updateSudokuData();

        }
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
        translate[0] -= dx * inv_scale;
        translate[1] += dy * inv_scale;
        u_translate.set(translate);
    }

    [mx, my] = [x, y];
});

document.addEventListener("mouseup", (ev) => {
    if (dragging) {
        clicked = false;
        dragging = false;
        return;
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
    return [cx, cy]
}

let td = 0; // distance between touches (for pinch zoom)
let tid = 0; // touch ID for moving
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
    translate[0] -= dx * inv_scale;
    translate[1] += dy * inv_scale;
    u_translate.set(translate);

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
    let bx = (mx - 0.5 * window.innerWidth * pixel_ratio) * inv_scale + translate[0];
    let by = (my - 0.5 * window.innerHeight * pixel_ratio) * inv_scale - translate[1] + 1;
    u_mouse_coords.set([bx, by]);
    // TODO: don't change highlighted cell when cell is selected for number input

    gl.draw();

    requestAnimationFrame(tick);
}
requestAnimationFrame(tick);
