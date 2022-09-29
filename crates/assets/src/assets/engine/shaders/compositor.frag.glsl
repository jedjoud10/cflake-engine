#version 460 core
out vec4 frag;

// Global settings indeed
uniform ivec2 resolution;
uniform float z_near;
uniform float z_far;

// Textures that we will sample
uniform sampler2D color;
uniform sampler2D depth;
uniform sampler2D shadow_map;

uniform float time;

// Post-processing compositor settings
uniform float tonemapping_strength;
uniform float exposure;
uniform float gamma;
uniform float vignette_strength;
uniform float vignette_size;

// Turn the 0-1 depth value to the zNear - zFar range
float linearize_depth(float d,float zNear,float zFar)
{
	d = 2.0 * d - 1.0;
    return zNear * zFar / (zFar + d * (zNear - zFar));
}

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
	sampled *= exposure;
	sampled = mix(sampled, aces(sampled), tonemapping_strength);
	sampled = pow(sampled, vec3(1.0 / gamma));

	// Create a simple vignette
	float vignette = length(abs(uv - 0.5));
	vignette += vignette_size;
	vignette = clamp(vignette, 0, 1);
	vignette = pow(vignette, vignette_strength);
	sampled = mix(sampled, vec3(0), vignette);

	// Sample the depth texture
	float depth = linearize_depth(texture(depth, uv).r, z_near, z_far);
	frag = vec4(sampled, 1.0);
}