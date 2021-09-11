#version 460 core
layout(location = 0) in vec2 vertex_pos;
layout(location = 1) in vec2 vertex_uvs;
uniform vec2 size;
uniform vec2 position;
uniform float depth;
out vec2 uvs;

void main() {
	vec2 position = vertex_pos.xy;
	position *= size;
	position += position;
	gl_Position = vec4(position, depth, 1);
	uvs = vertex_uvs;
}