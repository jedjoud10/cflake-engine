#version 460 core
#load general
#include "defaults/shaders/rendering/sky.func.glsl"
#include "defaults/shaders/rendering/sun.func.glsl"
#include "defaults/shaders/rendering/shadow_calculations.func.glsl"
#include "defaults/shaders/rendering/lighting.func.glsl"
layout(location = 0) out vec3 color;
uniform sampler2D diffuse_texture; // 0
uniform sampler2D emissive_texture; // 1
uniform sampler2D normals_texture; // 2
uniform sampler2D position_texture; // 3
uniform sampler2D depth_texture; // 4
uniform sampler2D sky_gradient; // 5
uniform sampler2DShadow shadow_map; // 6
uniform vec3 sunlight_dir;
uniform mat4 lightspace_matrix;
uniform float sunlight_strength;
uniform mat4 pr_matrix;
uniform mat4 pv_matrix;
uniform vec2 nf_planes;
in vec2 uvs;

void main() {
	ivec2 pixel = ivec2(uvs * _resolution);
	// Sample the textures
	vec3 normal = normalize(texture(normals_texture, uvs).xyz);
	vec3 diffuse = texture(diffuse_texture, uvs).xyz;
	vec3 emissive = texture(emissive_texture, uvs).xyz;
	vec3 position = texture(position_texture, uvs).xyz;
	// Calculate the dot product using the sun's direction vector and the up vector
	float sun_dot_product = dot(sunlight_dir, vec3(0, 1, 0));
	float time_of_day = sun_dot_product * 0.5 + 0.5;
	float sun_strength_factor = calculate_sun_strength(time_of_day);	
	vec3 pixel_dir = normalize((inverse(pr_matrix) * vec4(uvs * 2 - 1, 0, 1)).xyz);

	// Get fragment depth
	float odepth = texture(depth_texture, uvs).x;
	// Depth test with the sky
	if (odepth == 1.0) {
		// Sky gradient texture moment
		float sky_uv_sampler = dot(pixel_dir, vec3(0, 1, 0));
		color = calculate_sky_color(sky_gradient, sky_uv_sampler, time_of_day);
		color += max(pow(dot(pixel_dir, normalize(sunlight_dir)), 1024), 0) * sun_strength_factor;
	} else {
		// Shadow mapping calculations
		float in_shadow = calculate_shadows(position, normal, sunlight_dir, lightspace_matrix, shadow_map);
		// Calculate lighting
		vec3 frag_color = compute_lighting(sunlight_dir, sun_strength_factor * sunlight_strength, diffuse, normal, emissive, position, pixel_dir, in_shadow, sky_gradient, time_of_day);
		color = frag_color;
	}
}