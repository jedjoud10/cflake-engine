#version 460 core
layout(location = 0) in vec2 vertex_pos;
layout(location = 1) in vec2 vertex_uvs;
uniform vec2 size;
uniform vec2 offset_position;
uniform float depth;
out vec2 uvs;
// Map some value from a specific range to another range
float map(float x, float ra, float rb, float r2a, float r2b) {
    // https://stackoverflow.com/questions/3451553/value-remapping
    return r2a + (x - ra) * (r2b - r2a) / (rb - ra);
}
void main() {
	// Turn the -1, 1 range to 0, 1 range
	vec2 position = ((vertex_pos.xy) + 1) / 2;
	position *= size;
	position += offset_position;
	// Turn the 0, 1 back to the -1, 1 range
	position = position * 2 - 1; 
	gl_Position = vec4(position, depth, 1);
	uvs = vertex_uvs;
}