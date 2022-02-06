#version 460 core
#load general
#include "defaults\shaders\rendering\sky.func.glsl"
#include "defaults\shaders\rendering\sun.func.glsl"
out vec3 color;
uniform sampler2D diffuse_texture; // 0
uniform sampler2D emissive_texture; // 1
uniform sampler2D normals_texture; // 2
uniform sampler2D position_texture; // 3
uniform sampler2D depth_texture; // 4
uniform sampler2D default_sky_gradient; // 5
uniform sampler2D shadow_map; // 6


uniform vec3 directional_light_dir;
uniform float directional_light_strength;
uniform mat4 custom_vp_matrix;
uniform vec2 nf_planes;
uniform int debug_view;
uniform vec3 camera_pos;
uniform vec3 camera_dir;
uniform ivec2 resolution;
uniform float time;
in vec2 uv_coordinates;
uniform float test;

void main() {
	vec2 uvs = uv_coordinates;
	ivec2 pixel = ivec2(uv_coordinates * resolution);
	// Sample the textures
	vec3 normal = normalize(texture(normals_texture, uvs).xyz);
	vec3 diffuse = texture(diffuse_texture, uvs).xyz;
	vec3 emissive = texture(emissive_texture, uvs).xyz;
	vec3 position = texture(position_texture, uvs).xyz;
	// Calculate the dot product using the sun's direction vector and the up vector
	float sun_dot_product = dot(directional_light_dir, vec3(0, 1, 0));
	float sun_up_factor = sun_dot_product * 0.5 + 0.5;
	float sun_strength = calculate_sun_strength(sun_up_factor);
	
	
	// Calculate the diffuse lighting
	float light_val = max(dot(normal, normalize(directional_light_dir)), 0) * directional_light_strength * sun_strength;

	// Sky gradient texture moment
    vec3 pixel_dir = normalize((inverse(custom_vp_matrix) * vec4(uvs * 2 - 1, 0, 1)).xyz);
	float sky_uv_sampler = dot(pixel_dir, vec3(0, 1, 0));
	vec3 sky_color = calculate_sky_color(default_sky_gradient, sky_uv_sampler, sun_up_factor);
	
	// Add the sun
	sky_color += max(pow(dot(pixel_dir, normalize(directional_light_dir)), 512), 0) * sun_strength;

	// Used for ambient lighting
	float ambient_lighting_strength = 0.05;
	float sky_light_val = dot(normal, vec3(0, 1, 0)); 
	vec3 ambient_lighting_color = calculate_sky_color(default_sky_gradient, sky_light_val, sun_up_factor).xyz;
	
	// Add everything
	vec3 ambient_lighting = ambient_lighting_color * ambient_lighting_strength;
	vec3 frag_color = ambient_lighting;
	frag_color += diffuse * (light_val + 0.1);
	
	// Calculate some specular
	/*
	float specular_val = pow(clamp(dot(pixel_dir, reflect(directional_light_dir, normal)), 0, 1), 256);
	frag_color += specular_val * 0.5;
	*/	
	// Test emissive
	if (all(notEqual(emissive, vec3(0, 0, 0)))) {
		frag_color = emissive;
	}

	// Calculate linear depth
	float odepth = texture(depth_texture, uvs).x;
	float depth = (nf_planes.x * odepth) / (nf_planes.y - odepth * (nf_planes.y - nf_planes.x));	

	// Shadow mapping calculations
	float shadow_mapped_depth = texture(shadow_map, uvs * 2.0).r;

	// Depth test with the sky
	if (depth == 1.0) {
		color = sky_color;
	} else {
		color = frag_color;
	}

	if (uvs.x < 0.5 && uvs.y < 0.5) {
		color = vec3(shadow_mapped_depth);
	}
}