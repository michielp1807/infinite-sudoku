#version 100

precision highp float;

varying vec2 v_uv;

// uniform vec2 u_mouse_coords;
uniform sampler2D u_numbers_texture;
uniform sampler2D u_sudoku;
uniform float u_inv_scale;

const float THIRD = 1.0 / 3.0;

void main() {
	vec2 uv = mod(v_uv, 12.0) / 9.0;
	if(uv.x > 1.0 || uv.y > 1.0) {
		uv = mod(v_uv + 6.0, 12.0) / 9.0;
	}

	int number = int(255.0 * texture2D(u_sudoku, uv));

	vec2 num_uv = mod(v_uv, 1.0) * THIRD;
	num_uv += vec2(THIRD * mod(float(number - 1), 3.0), THIRD * float((number - 1) / 3));
	vec4 num = texture2D(u_numbers_texture, num_uv);

	float num_normal = 1.0 - num.r;
	float num_bold = 1.0 - num.g;

	float grid_blur = 1.0 * u_inv_scale;

	// vec3 color = vec3(block_border);
	// vec3 color = vec3(uv, 0.0);
	vec3 color = vec3(1.0);

	// TODO: highlight hovered sudoku(s), row(s), and column(s)
	// TODO: highlight errors (put extra bits in sudoku texture)

	if(uv.x > 1.0 || uv.y > 1.0) {
		color = vec3(0.8); // gray background
	} else {
		color = mix(vec3(0.0), color, num_normal); // add numbers

		float grid_thickness = 0.01;
		float grid_opacity = 1.0 - min(1.0, 7.5 * u_inv_scale);
		vec2 cell = mod(uv * 9.0, 1.0);
		float cell_border = min(1.0 - max(cell.x, cell.y), min(cell.x, cell.y));
		cell_border = smoothstep(grid_thickness + grid_blur, grid_thickness, cell_border);
		color = mix(color, vec3(0.0), grid_opacity * cell_border);

	}

	float grid_thickness = 0.03;
	float grid_opacity = 1.0 - min(1.0, 10.0 * u_inv_scale);
	vec2 block = mod(uv * 3.0, 1.0);
	float block_border = 3.0 * min(1.0 - max(block.x, block.y), min(block.x, block.y));
	block_border = smoothstep(grid_thickness + grid_blur, grid_thickness, block_border);
	color = mix(color, vec3(0.0), grid_opacity * block_border);

	gl_FragColor = vec4(color, 1.0);
}
