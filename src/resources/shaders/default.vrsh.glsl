#version 460 core
layout(location = 0) in vec3 vertex_pos;
layout(location = 1) in vec3 vertex_normal;
layout(location = 2) in vec3 vertex_tangent;
layout(location = 3) in vec2 vertex_uv;
uniform mat4 mvp_matrix;
out vec3 debug_color;
out vec3 normal;

void main() {
	gl_Position = mvp_matrix * vec4(vertex_pos, 1.0);
	normal = normalize(mvp_matrix * (vec4(vertex_normal, 0.0)).xyz);
	debug_color = normalize((vec4(vertex_normal, 1.0)).xyz);
	debug_color = vertex_tangent;
}