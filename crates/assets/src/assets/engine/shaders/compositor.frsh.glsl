#version 460 core
out vec4 frag;

// Global settings indeed
uniform ivec2 resolution;

// Textures that we will sample
uniform sampler2D color;

// Post-processing compositor settings
uniform float tonemapping_strength;
uniform float exposure;
uniform float vignette_strength;
uniform float vignette_size;

void main() {
	vec2 tex_coord = gl_FragCoord.xy / vec2(resolution);
	
	// Sample the color texture and apply gamma correction
	vec3 sampled = texture(color, tex_coord).xyz;
	float gamma = 2.2;
	vec3 final = pow(sampled, vec3(1.0 / gamma)); 

	frag = vec4(final, 1.0);
}