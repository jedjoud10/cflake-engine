#version 460 core
layout(location = 0) in vec3 vertex_pos;
uniform mat4 mvp_matrix;
out vec3 normal;

void main() {
	gl_Position = mvp_matrix * vec4(vertex_pos, 1.0);
	normal = normalize(vertex_pos);
}