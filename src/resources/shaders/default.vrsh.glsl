#version 430 core
layout(location = 0) in vec3 vertex_pos;
uniform mat4 mvp_matrix;

void main() {
	gl_Position = mvp_matrix * vec4(vertex_pos, 1.0);
}