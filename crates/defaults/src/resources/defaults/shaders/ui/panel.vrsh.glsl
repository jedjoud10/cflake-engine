#version 460 core
layout(location = 0) in vec2 vertex_pos;
layout(location = 1) in vec4 vertex_uvs;
layout(location = 2) in vec4 vertex_texture_uvs;
layout(location = 3) in float vertex_depths;
layout(location = 4) in vec3 vertex_colors;
out vec4 v_screen_uvs;
out vec4 v_texture_uvs;
out vec3 v_color;
void main() {
	gl_Position = vec4(vertex_pos, 0.5, 1);
}