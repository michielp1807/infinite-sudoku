#version 300 es

precision highp float;

in vec2 a_position;
out vec2 v_uv;

uniform vec2 u_window_resolution;
uniform float u_inv_scale;
uniform vec2 u_translate;

void main() {
	v_uv = a_position * 0.5 * u_window_resolution * u_inv_scale + u_translate;
	v_uv.y = 1.0 - v_uv.y;
	gl_Position = vec4(a_position, 0.0, 1.0);
}
