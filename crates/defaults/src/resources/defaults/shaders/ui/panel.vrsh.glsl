#version 460 core
layout(location = 0) in vec2 vertex_pos;
layout(location = 1) in vec2 vertex_screen_uvs;
layout(location = 2) in vec2 vertex_texture_uvs;
layout(location = 3) in float vertex_depth;
layout(location = 4) in vec4 vertex_color;
out vec2 v_screen_uvs;
out vec2 v_texture_uvs;
out vec4 v_color;
void main() {
	v_screen_uvs = vertex_screen_uvs;
	v_texture_uvs = vertex_texture_uvs;
	v_color = vertex_color;
	gl_Position = vec4(vertex_pos, 0.5, 1);
}