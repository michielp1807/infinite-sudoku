#version 300 es

precision highp float;

in vec2 v_uv;
out vec4 frag_color;

uniform vec2 u_mouse_coords;
uniform vec2 u_selected_cell;
uniform int u_selected_value;
uniform sampler2D u_numbers_texture;
uniform sampler2D u_sudoku;
uniform float u_inv_scale;
uniform vec2 u_world_size;

const float THIRD = 1.0 / 3.0;

// TODO: add dark mode?
const vec3 col_num = vec3(0.0);
const vec3 col_selected_num = vec3(0.0, 0.22, 0.58);
const vec3 col_cell = vec3(1.0);
const vec3 col_selected_cell = vec3(0.37, 0.74, 1.0);
const vec3 col_hover = vec3(0.0);
const vec3 col_border = vec3(0.0);
const vec3 col_unused_block = vec3(0.83);

// prevent floating point errors by rounding values
vec2 rounded_mod(vec2 a, vec2 b) {
	vec2 m = a - floor((a + 0.5) / b) * b;
	return floor(m + 0.5);
}

void main() {
	float grid_blur = 1.0 * u_inv_scale;

	vec3 color = vec3(col_cell);

	// the sudoku pattern repeats in 12x12 sections of 4x4 blocks
	vec2 b4x4 = mod(v_uv, 12.0) / 3.0;
	vec2 fb4x4 = floor(b4x4);

	float top_left = float(fb4x4.x == 0.0 && fb4x4.y < 2.0);
	float top_right = float(fb4x4.x >= 2.0 && fb4x4.y < 2.0);
	float bottom_right = float(fb4x4.x == 3.0 && fb4x4.y == 3.0);

	// sudoku coords specify which sudoku is here (on a diagonal grid)
	vec2 sudoku_coord = floor(v_uv / 12.0);
	sudoku_coord = vec2(sudoku_coord.x - sudoku_coord.y, -sudoku_coord.x - sudoku_coord.y);
	sudoku_coord += vec2(0.0, 1.0) * top_left;
	sudoku_coord += vec2(1.0, 0.0) * top_right;
	sudoku_coord += vec2(0.0, -1.0) * bottom_right;

	// 0..9 sudoku uv (regular coordinates)
	vec2 sudoku_uv = v_uv - vec2(0.0, 3.0)
		+ vec2(6.0, 6.0) * top_left
		+ vec2(-6.0, 6.0) * top_right
		+ vec2(-6.0, -6.0) * bottom_right;
	sudoku_uv = mod(sudoku_uv, 12.0);

	// covert sudoku uv to number index
	vec2 block_uv = mod(v_uv, 3.0);
	vec2 fblock_uv = floor(block_uv);
	float cell_index = fblock_uv.x + fblock_uv.y * 3.0;
	vec2 fsudoku_uv = floor(sudoku_uv * THIRD);
	float block_index = max(fsudoku_uv.x + fsudoku_uv.y * 3.0 - 2.0, 0.0);
	float index = 9.0 * block_index + cell_index;

	// TODO: highlight errors (put extra bits in sudoku texture)

	// highlight selected sudoku(s)
	vec2 selected_fb4x4 = floor(mod(u_selected_cell, 12.0) / 3.0);
	vec2 selected_sudoku = floor(u_selected_cell / 12.0);
	selected_sudoku = vec2(selected_sudoku.x - selected_sudoku.y, -selected_sudoku.x - selected_sudoku.y);
	selected_sudoku += vec2(0.0, 1.0) * float(selected_fb4x4.x == 0.0 && selected_fb4x4.y < 2.0);
	selected_sudoku += vec2(1.0, 0.0) * float(selected_fb4x4.x >= 2.0 && selected_fb4x4.y < 2.0);
	selected_sudoku += vec2(0.0, -1.0) * float(selected_fb4x4.x == 3.0 && selected_fb4x4.y == 3.0);
	vec2 selected_sudoku2 = selected_sudoku;
	selected_sudoku += vec2(0.0, -1.0) * float(
		(selected_fb4x4.x == 0.0 && selected_fb4x4.y == 1.0) || (selected_fb4x4.x == 2.0 && selected_fb4x4.y == 3.0)
	);
	selected_sudoku += vec2(-1.0, 0.0) * float(
		(selected_fb4x4.x == 0.0 && selected_fb4x4.y == 3.0) || (selected_fb4x4.x == 2.0 && selected_fb4x4.y == 1.0)
	);
	vec2 sudoku_coord2 = sudoku_coord;
	sudoku_coord2 += vec2(0.0, -1.0) * float((fb4x4.x == 0.0 && fb4x4.y == 1.0) || (fb4x4.x == 2.0 && fb4x4.y == 3.0));
	sudoku_coord2 += vec2(-1.0, 0.0) * float((fb4x4.x == 0.0 && fb4x4.y == 3.0) || (fb4x4.x == 2.0 && fb4x4.y == 1.0));

	bool is_hovered_sudoku = selected_sudoku == sudoku_coord || selected_sudoku2 == sudoku_coord || selected_sudoku == sudoku_coord2 || selected_sudoku2 == sudoku_coord2;
	is_hovered_sudoku = is_hovered_sudoku && !((selected_fb4x4.x == 1.0 && selected_fb4x4.y == 0.0) || (selected_fb4x4.x == 3.0 && selected_fb4x4.y == 2.0));
	color = mix(color, col_selected_cell, 0.3 * float(is_hovered_sudoku));

	// highlight selected row/column/block
	vec2 cell_coord = floor(v_uv);
	vec2 selected_cell = floor(u_selected_cell);
	vec2 block_coord = floor(v_uv / 3.0);
	vec2 selected_block = floor(u_selected_cell / 3.0);
	color = mix(color, col_selected_cell, 0.4 * float(is_hovered_sudoku &&
		(selected_cell.x == cell_coord.x || selected_cell.y == cell_coord.y || block_coord == selected_block)));

	// highlight selected cell
	color = mix(color, col_selected_cell, float(selected_cell == cell_coord));

	// add number in cell
	sudoku_coord = rounded_mod(sudoku_coord, u_world_size);
	float sudoku_index = sudoku_coord.x + sudoku_coord.y * u_world_size.x;
	sudoku_index /= u_world_size.x * u_world_size.y - 1.0;
	int cell_data = int(255.0 * texture(u_sudoku, vec2(index / (7.0 * 9.0 - 1.0), sudoku_index)));
	bool user_entered = (cell_data & 16) > 0;

	int number = cell_data & 15;
	vec2 num_block_uv = (mod(v_uv / 3.0, 1.0) - 0.5) * 1.01 + 0.5; // slightly scale down blocks to account for thicker border
	vec2 num_uv = mod(num_block_uv * 3.0, 1.0) * THIRD;
	num_uv += vec2(THIRD * mod(float(number - 1), 3.0), THIRD * float((number - 1) / 3));

	vec4 num = texture(u_numbers_texture, num_uv);
	float num_bolder = num.r;
	float num_bold = num.g;
	float num_normal = num.b;

	float num_text = mix(num_bold, num_normal, float(user_entered));

	vec3 num_color = mix(col_num, col_selected_num, float(number == u_selected_value));

	color = mix(color, num_color, num_text * float(number > 0 && number <= 9));

	// highlight hovered cell
	color = mix(color, col_hover, 0.1 * float(floor(u_mouse_coords) == cell_coord));

	// add cell grid
	float grid_thickness = 0.01;
	float grid_opacity = max(1.0 - min(1.0, 5.0 * u_inv_scale), 0.02);
	vec2 cell_uv = mod(v_uv, 1.0);
	float cell_border = min(1.0 - max(cell_uv.x, cell_uv.y), min(cell_uv.x, cell_uv.y));
	cell_border = smoothstep(grid_thickness + grid_blur, grid_thickness, cell_border);
	color = mix(color, col_border, grid_opacity * cell_border);

	// gray background between sudokus
	color = mix(color, col_unused_block, float(
		(fb4x4.x == 1.0 && fb4x4.y == 0.0) || (fb4x4.x == 3.0 && fb4x4.y == 2.0)
	));

	// add block grid
	float grid_thickness2 = 0.03;
	float grid_opacity2 = max(1.0 - min(1.0, 7.5 * u_inv_scale), 0.04);
	block_uv = block_uv / 3.0;
	float block_border = 3.0 * min(1.0 - max(block_uv.x, block_uv.y), min(block_uv.x, block_uv.y));
	block_border = smoothstep(grid_thickness2 + grid_blur, grid_thickness2, block_border);
	color = mix(color, col_border, grid_opacity2 * block_border);

	frag_color = vec4(color, 1.0);
}
