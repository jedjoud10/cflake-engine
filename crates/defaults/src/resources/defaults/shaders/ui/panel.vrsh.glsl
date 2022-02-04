#version 460 core
layout(location = 0) in vec2 vertex_pos;
layout(location = 1) in vec2 instance_panel_center;
layout(location = 2) in vec2 instance_panel_size;
layout(location = 3) in vec2 vertex_texture_uvs;
layout(location = 4) in float instance_depth;
layout(location = 5) in vec4 instance_color;
out vec2 v_texture_uvs;
out vec4 v_color;
void main() {
	// The vertex_panel_center range is in between 0 and 1, so we must convert it to a range between -1 and 1 instead
	vec2 offset = (instance_panel_center * 2.0) - 1.0;
	v_texture_uvs = (vertex_pos + 1) / 2.0;
	v_color = instance_color;
	gl_Position = vec4(vertex_pos * instance_panel_size + offset, instance_depth, 1);
}