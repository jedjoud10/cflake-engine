#version 460 core
out vec4 frag;

// Textures that we will sample
uniform texture2D image;

// Post-processing compositor settings
uniform float tonemapping_strength;
uniform float exposure;
uniform float vignette_strength;
uniform float vignette_size;

void main() {
	vec2 tex_coord = gl_FragCoord.xy;
	vec3 sampled = texture(image, tex_coord).xyz;
	frag = vec4(sampled, 1.0);
}