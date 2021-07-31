#version 460 core
layout(location = 0) in vec3 vertex_pos;
layout(location = 1) in vec3 vertex_normal;
layout(location = 2) in vec3 vertex_tangent;
layout(location = 3) in vec2 vertex_uv;
uniform mat4 mvp_matrix;
uniform mat4 model_matrix;
out vec3 debug_color;
out vec3 normal;
out vec3 world_position;

void main() {
	vec4 mvp_pos = mvp_matrix * vec4(vertex_pos, 1.0);
	vec4 model_pos = model_matrix * vec4(vertex_pos, 1.0);
	gl_Position = mvp_pos;
	world_position = model_pos.xyz;
	normal = normalize((vec4(vertex_normal, 0.0)).xyz);
	debug_color = vec3(vertex_uv, 0.0);
}