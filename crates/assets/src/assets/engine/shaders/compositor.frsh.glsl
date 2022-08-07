#version 460 core
out vec4 frag;

// Textures that we will sample
uniform sampler2D color;

// Post-processing compositor settings
uniform float tonemapping_strength;
uniform float exposure;
uniform float vignette_strength;
uniform float vignette_size;

void main() {
	vec2 tex_coord = gl_FragCoord.xy;
	vec3 sampled = texture(color, tex_coord).xyz;
	frag = vec4(1, 1, 1, 1.0);
}