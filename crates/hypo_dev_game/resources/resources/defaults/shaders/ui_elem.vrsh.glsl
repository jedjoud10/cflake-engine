#version 460 core
layout(location = 0) in vec2 vertex_pos;
layout(location = 1) in vec2 vertex_uvs;
uniform vec2 f_size;
uniform vec2 f_offset_position;
uniform vec2 p_size;
uniform vec2 p_offset_position;
uniform ivec2 resolution;
uniform float depth;
out vec2 uvs;

void main() {
	// Position is in the -1, 1 range
	vec2 position = vertex_pos.xy;
	position *= f_size/2.0;
	// Position is in the 0, 1 range
	position += (f_offset_position*2)-1;
	gl_Position = vec4(position, depth, 1);
	uvs = vertex_uvs;
}