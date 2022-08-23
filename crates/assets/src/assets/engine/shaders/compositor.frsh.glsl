#version 460 core
out vec4 frag;

// Global settings indeed
uniform ivec2 resolution;

// Textures that we will sample
uniform sampler2D color;

// Post-processing compositor settings
uniform float tonemapping_strength;
uniform float exposure;
uniform float gamma;
uniform float vignette_strength;
uniform float vignette_size;

// Narkowicz 2015, "ACES Filmic Tone Mapping Curve"
vec3 aces(vec3 x) {
    const float a = 2.51;
    const float b = 0.03;
    const float c = 2.43;
    const float d = 0.59;
    const float e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}

void main() {
	vec2 uv = gl_FragCoord.xy / vec2(resolution);
	
	// Sample the color texture and apply gamma correction
	vec3 sampled = texture(color, uv).xyz;
	vec3 mapped = pow(sampled, vec3(1.0 / gamma));
	//vec3 mapped = mix(sampled, aces(sampled), tonemapping_strength);

	/*
	// Create a simple vignette
	float vignette = length(abs(uv - 0.5));
	vignette += vignette_size;
	vignette = clamp(vignette, 0, 1);
	vignette = pow(vignette, vignette_strength);
	mapped = mix(mapped, vec3(0), vignette);
	*/

	frag = vec4(mapped, 1.0);
}