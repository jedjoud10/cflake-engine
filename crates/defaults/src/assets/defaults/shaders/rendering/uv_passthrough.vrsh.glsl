#version 460 core
layout(location = 0) in vec3 mesh_pos;
layout(location = 3) in vec2 mesh_uv;
out vec2 uvs;

void main() {
	gl_Position = vec4(mesh_pos, 1);
	uvs = mesh_uv;
}