#version 460 core
#load general
#include "defaults/shaders/rendering/sky.func.glsl"
#include "defaults/shaders/rendering/sun.func.glsl"
#include "defaults/shaders/rendering/shadows.func.glsl"
#include "defaults/shaders/rendering/lighting.func.glsl"
#include "defaults/shaders/rendering/postprocessing.func.glsl"
out vec4 color;
uniform sampler2D diffuse_texture; // 0
uniform sampler2D emissive_texture; // 1
uniform sampler2D normals_texture; // 2
uniform sampler2D position_texture; // 3
uniform sampler2D depth_texture; // 4
uniform sampler2D sky_gradient; // 5
uniform sampler2D shadow_map; // 6
uniform vec3 sunlight_dir;
uniform mat4 lightspace_matrix;
uniform float sunlight_strength;
uniform mat4 inverse_pr_matrix;
uniform mat4 pv_matrix;
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
	vec3 pixel_dir = normalize((inverse_pr_matrix * vec4(uvs * 2 - 1, 0, 1)).xyz);

	// Get fragment depth
	vec3 final_color = vec3(0, 0, 0);
	float odepth = texture(depth_texture, uvs).x;
	// Depth test with the sky
	if (odepth == 1.0) {
		// Sky gradient texture moment
		float sky_uv_sampler = dot(pixel_dir, vec3(0, 1, 0));
		final_color = calculate_sky_color(sky_gradient, pixel_dir, sky_uv_sampler, time_of_day);
		final_color += max(pow(dot(pixel_dir, normalize(sunlight_dir)), 4096), 0) * sun_strength_factor * 40.0;
	} else {
		float in_shadow = calculate_shadows(position, normal, sunlight_dir, lightspace_matrix, shadow_map);
		final_color = compute_lighting(sunlight_dir, sun_strength_factor * sunlight_strength, diffuse, normal, emissive, position, pixel_dir, in_shadow, sky_gradient, time_of_day);
	}

	color = vec4(post_rendering(uvs, final_color), 1.0);
}