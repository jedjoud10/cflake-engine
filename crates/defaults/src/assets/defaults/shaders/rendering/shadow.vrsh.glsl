#version 460 core
layout(location = 0) in vec3 mesh_pos;
uniform mat4 lsm_matrix;

void main() {
	gl_Position = lsm_matrix * vec4(mesh_pos, 1.0);
}