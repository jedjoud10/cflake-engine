#version 460 core
layout(location = 0) in vec2 vertex_pos;
layout(location = 1) in vec2 vertex_uvs;
uniform vec2 size;
uniform float font_size;
uniform vec2 offset_position;
uniform vec2 min_padding;
uniform vec2 max_padding;
uniform float font_ratio;
uniform vec2 character_offset;
uniform vec2 resolution;
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
	// Gotta have resolution ratio compensation
	position *= size;
	// Scale the font
	position *= vec2(font_ratio, 1) * font_size;
	position += offset_position;	
	position += character_offset;
	// Turn the 0, 1 back to the -1, 1 range
	position = position * 2 - 1; 
    // Map the uvs to the min_padding and the max_padding
    vec2 new_uvs = vec2(map(vertex_uvs.x, 0, 1, min_padding.x, max_padding.x), map(1-vertex_uvs.y, 0, 1, min_padding.y, max_padding.y));
    new_uvs = vec2(new_uvs.x, new_uvs.y);
	gl_Position = vec4(position, depth, 1);
	uvs = new_uvs;
}