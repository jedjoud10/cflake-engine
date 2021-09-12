#version 460 core
layout(location = 0) in vec2 vertex_pos;
layout(location = 1) in vec2 vertex_uvs;
uniform vec2 size;
uniform vec2 offset_position;
uniform float depth;
out vec2 uvs;

void main() {
	// Position is in the -1, 1 range
	vec2 position = vertex_pos.xy;
	position *= size/2.0;
	// Position is in the 0, 1 range
	position += (offset_position*2)-1;
	gl_Position = vec4(position, depth, 1);
	uvs = vertex_uvs;
}