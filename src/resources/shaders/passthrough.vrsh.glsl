#version 460 core
layout(location = 0) in vec3 vertex_pos;
layout(location = 3) in vec2 vertex_uv;
out vec2 uv_coordinates;

void main() {
	gl_Position = vec4(vertex_pos.x, vertex_pos.y, 0, 1);
	uv_coordinates = vertex_uv;
}