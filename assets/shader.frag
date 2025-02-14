#version 100

precision highp float;

varying vec2 v_uv;

uniform vec2 u_mouse_coords;
uniform sampler2D u_numbers_texture;
uniform sampler2D u_sudoku;
uniform float u_inv_scale;

const float THIRD = 1.0 / 3.0;
const float TWO_THIRD = 2.0 / 3.0;
const vec3 HIGHLIGHT_COLOR = vec3(0.23, 0.69, 1.0);

vec2 get_sudoku_coord(vec2 uv) {
	vec2 diagonal = vec2(uv.x - uv.y + 6.0, uv.x + uv.y + 9.0);
	return floor(diagonal / 12.0);
}

vec2 get_sudoku_uv(vec2 uv, vec2 sudoku_coord) {
	float offset = mod(sudoku_coord.x + sudoku_coord.y, 2.0);
	return mod(mix(uv + 6.0, uv, offset), 12.0) / 9.0;
}

vec2 get_other_sudoku_coord(vec2 sudoku_coord, vec2 sudoku_uv) {
	vec2 other_sudoku_coord = sudoku_coord;
	other_sudoku_coord.x -= float(sudoku_uv.x < THIRD && sudoku_uv.y > TWO_THIRD);
	other_sudoku_coord.x += float(sudoku_uv.x > TWO_THIRD && sudoku_uv.y < THIRD);
	other_sudoku_coord.y -= float(sudoku_uv.x < THIRD && sudoku_uv.y < THIRD);
	other_sudoku_coord.y += float(sudoku_uv.x > TWO_THIRD && sudoku_uv.y > TWO_THIRD);
	return other_sudoku_coord;
}

void main() {
	float grid_blur = 1.0 * u_inv_scale;

	vec3 color = vec3(1.0);

	vec2 sudoku_coord = get_sudoku_coord(v_uv);
	vec2 sudoku_uv = get_sudoku_uv(v_uv, sudoku_coord);
	vec2 sudoku_coord2 = get_other_sudoku_coord(sudoku_coord, sudoku_uv);

	// TODO: highlight errors (put extra bits in sudoku texture)

	vec2 m_sudoku_coord = get_sudoku_coord(u_mouse_coords);
	vec2 m_sudoku_uv = get_sudoku_uv(u_mouse_coords, m_sudoku_coord);
	vec2 m_sudoku_coord2 = get_other_sudoku_coord(m_sudoku_coord, m_sudoku_uv);

	bool is_hovered_sudoku = m_sudoku_coord == sudoku_coord || m_sudoku_coord == sudoku_coord2 || m_sudoku_coord2 == sudoku_coord || m_sudoku_coord2 == sudoku_coord2;
	is_hovered_sudoku = is_hovered_sudoku && !(m_sudoku_uv.x > 1.0 || m_sudoku_uv.y > 1.0);

	// highlight currently hovered sudoku(s)
	color = mix(color, HIGHLIGHT_COLOR, 0.2 * float(is_hovered_sudoku));

	// highlight hovered row/column/block
	vec2 cell_coord = floor(v_uv);
	vec2 mouse_cell = floor(u_mouse_coords);
	vec2 block = floor(v_uv / 3.0);
	vec2 mouse_block = floor(u_mouse_coords / 3.0);
	color = mix(color, HIGHLIGHT_COLOR, 0.4 * float(is_hovered_sudoku &&
		(mouse_cell.x == cell_coord.x || mouse_cell.y == cell_coord.y || block == mouse_block)));

	// highlight hovered cell
	color = mix(color, HIGHLIGHT_COLOR, float(mouse_cell == cell_coord));

	// add number in cell
	int number = int(255.0 * texture2D(u_sudoku, sudoku_uv));
	vec2 num_uv = mod(v_uv, 1.0) * THIRD;
	num_uv += vec2(THIRD * mod(float(number - 1), 3.0), THIRD * float((number - 1) / 3));

	vec4 num = texture2D(u_numbers_texture, num_uv);
	float num_normal = num.r;
	float num_bold = num.g;

	color = mix(color, vec3(0.0), num_normal * float(number > 0 && number <= 9));

	// add cell grid
	float grid_thickness = 0.01;
	float grid_opacity = 1.0 - min(1.0, 7.5 * u_inv_scale);
	vec2 cell_uv = mod(sudoku_uv * 9.0, 1.0);
	float cell_border = min(1.0 - max(cell_uv.x, cell_uv.y), min(cell_uv.x, cell_uv.y));
	cell_border = smoothstep(grid_thickness + grid_blur, grid_thickness, cell_border);
	color = mix(color, vec3(0.0), grid_opacity * cell_border);

	// gray background between sudokus
	color = mix(color, vec3(0.83), float(sudoku_uv.x > 1.0 || sudoku_uv.y > 1.0));

	// add block grid
	grid_thickness = 0.03;
	grid_opacity = 1.0 - min(1.0, 10.0 * u_inv_scale);
	vec2 block_uv = mod(sudoku_uv * 3.0, 1.0);
	float block_border = 3.0 * min(1.0 - max(block_uv.x, block_uv.y), min(block_uv.x, block_uv.y));
	block_border = smoothstep(grid_thickness + grid_blur, grid_thickness, block_border);
	color = mix(color, vec3(0.0), grid_opacity * block_border);

	gl_FragColor = vec4(color, 1.0);
}
