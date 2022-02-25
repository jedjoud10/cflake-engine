#version 460 core
layout(location = 0) in vec3 mesh_pos;
layout(location = 1) in vec3 mesh_normal;
layout(location = 2) in vec4 mesh_tangent;
layout(location = 3) in vec2 mesh_uv;
layout(location = 4) in vec3 mesh_color;
out vec2 uvs;

void main() {
	gl_Position = vec4(mesh_pos, 1);
	uvs = mesh_uv;
}