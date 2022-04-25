#version 460 core
layout(location = 0) in vec3 mesh_pos;
uniform mat4 matrix;

out vec3 local_pos;

void main() {
	gl_Position = matrix * vec4(mesh_pos, 1.0);
	local_pos = mesh_pos;
}