#version 460 core
out vec4 out_color;
in vec2 uvs;
uniform vec2 min_padding;
uniform vec2 max_padding;
uniform sampler2D atlas_texture;
uniform vec4 color;
// Map some value from a specific range to another range
float map(float x, float ra, float rb, float r2a, float r2b) {
    // https://stackoverflow.com/questions/3451553/value-remapping
    return r2a + (x - ra) * (r2b - r2a) / (rb - ra);
}
void main() {
    out_color = vec4(uvs, 0, 1);
    // Map the uvs to the min_padding and the max_padding
    vec2 new_uvs = vec2(map(uvs.x, 0, 1, min_padding.x, max_padding.x), map(uvs.y, 0, 1, min_padding.y, max_padding.y));
    // Get the color of this specific pixel in the font atlas
    float signed_distance = texture(atlas_texture, uvs).x;
	out_color = vec4(signed_distance, signed_distance, signed_distance, 1);
}