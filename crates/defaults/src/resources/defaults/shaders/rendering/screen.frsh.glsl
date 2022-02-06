#version 460 core
#load general
#include "defaults\shaders\rendering\sky.func.glsl"
#include "defaults\shaders\rendering\sun.func.glsl"
#include "defaults\shaders\rendering\shadow_calculations.func.glsl"
out vec3 color;
uniform sampler2D diffuse_texture; // 0
uniform sampler2D emissive_texture; // 1
uniform sampler2D normals_texture; // 2
uniform sampler2D position_texture; // 3
uniform sampler2D depth_texture; // 4
uniform sampler2D default_sky_gradient; // 5
uniform sampler2DShadow shadow_map; // 6


uniform vec3 directional_light_dir;
uniform mat4 lightspace_matrix;
uniform float directional_light_strength;
uniform mat4 projection_rotation_matrix;
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
    vec3 pixel_dir = normalize((inverse(projection_rotation_matrix) * vec4(uvs * 2 - 1, 0, 1)).xyz);
	float sky_uv_sampler = dot(pixel_dir, vec3(0, 1, 0));
	vec3 sky_color = calculate_sky_color(default_sky_gradient, sky_uv_sampler, sun_up_factor);
	
	// Add the sun
	sky_color += max(pow(dot(pixel_dir, normalize(directional_light_dir)), 1024), 0) * sun_strength;

	// Used for ambient lighting
	float ambient_lighting_strength = 0.1;
	float sky_light_val = dot(normal, vec3(0, 1, 0)); 
	vec3 ambient_lighting_color = calculate_sky_color(default_sky_gradient, sky_light_val, sun_up_factor).xyz;
	
	// Shadow mapping calculations
	float in_shadow = calculate_shadows(position, normal, directional_light_dir, lightspace_matrix, shadow_map);

	// Add everything
	vec3 ambient_lighting = ambient_lighting_color * ambient_lighting_strength + diffuse * 0.1;
	vec3 frag_color = ambient_lighting;
	frag_color += (1 - in_shadow) * (diffuse * light_val);
	
	// Calculate some specular
	/*
	float specular_val = pow(clamp(dot(pixel_dir, reflect(directional_light_dir, normal)), 0, 1), 128);
	frag_color += specular_val * 0.5 * (1 - in_shadow);
	*/
	// Test emissive
	if (all(notEqual(emissive, vec3(0, 0, 0)))) {
		frag_color = emissive;
	}

	// Calculate linear depth
	float odepth = texture(depth_texture, uvs).x;
	float depth = (nf_planes.x * odepth) / (nf_planes.y - odepth * (nf_planes.y - nf_planes.x));	

	// Depth test with the sky
	if (depth == 1.0) {
		color = sky_color;
	} else {
		color = frag_color;
	}
}