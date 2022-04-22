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
uniform sampler2D tangents_texture; // 2
uniform sampler2D bitangents_texture; // 3
uniform sampler2D position_texture; // 4
uniform sampler2D depth_texture; // 5
uniform sampler2D sky_gradient; // 6
uniform sampler2D shadow_map; // 7
uniform vec3 sunlight_dir;
uniform mat4 lightspace_matrix;
uniform float sunlight_strength;
uniform mat4 inverse_pr_matrix;
uniform mat4 pv_matrix;
in vec2 uvs;


void main() {
	ivec2 pixel = ivec2(uvs * _resolution);
	// Sample the textures
	vec3 tangent = texture(tangents_texture, uvs).rgb;
	vec3 bitangent = texture(bitangents_texture, uvs).rgb;
	vec3 diffuse = texture(diffuse_texture, uvs).rgb;
	vec3 emissive = texture(emissive_texture, uvs).rgb;
	vec3 position = texture(position_texture, uvs).rgb;

	// Reconstruct the world normal using the tangent and bitangent
	vec3 normal = cross(tangent, bitangent);

	/*
	vec3 normal = texture(model_normal, (m_uv) * uv_scale).xyz * 2.0 - 1.0;
	tangent_space_normals.xy *= bumpiness;
	frag_normal = normalize(tbn * tangent_space_normals);
	frag_tangent = vec4(0);

	vec3 bitangent = mesh_tangent.w * cross(mesh_tangent.xyz, mesh_normal);
	m_tangents = vec4(normalize((mesh_matrix * vec4(mesh_tangent.xyz, 0.0)).xyz), mesh_tangent.w);
	vec3 t = m_tangents.xyz;
	vec3 b = normalize((mesh_matrix * vec4(bitangent, 0.0)).xyz);
	vec3 n = m_normal;
	tbn = mat3(t, b, n);
	*/


	// Calculate the dot product using the sun's direction vector and the up vector
	float sun_dot_product = dot(sunlight_dir, vec3(0, 1, 0));
	float time_of_day = sun_dot_product * 0.5 + 0.5;
	float sun_strength_factor = calculate_sun_strength(time_of_day);	
	vec3 pixel_dir = normalize((inverse_pr_matrix * vec4(uvs * 2 - 1, 0, 1)).xyz);

	// Get fragment depth
	vec3 final_color = tangent;
	/*
	float odepth = texture(depth_texture, uvs).x;
	// Depth test with the sky
	if (odepth == 1.0) {
		// Sky gradient texture moment
		float sky_uv_sampler = dot(pixel_dir, vec3(0, 1, 0));
		final_color = calculate_sky_color(sky_gradient, pixel_dir, sky_uv_sampler, time_of_day);
		final_color += max(pow(dot(pixel_dir, normalize(sunlight_dir)), 4096), 0) * sun_strength_factor * 40;
	} else {
		// Shadow map
		float in_shadow = calculate_shadows(position, normal, sunlight_dir, lightspace_matrix, shadow_map);

		// Normal mapping shadows
		//float in_shadow_normals = calculate_shadows_normal_map(position, sunlight_dir, normals_texture, pv_matrix);

		final_color = compute_lighting(sunlight_dir, sun_strength_factor * sunlight_strength, diffuse, normal, emissive, position, pixel_dir, in_shadow, sky_gradient, time_of_day);
		//final_color = vec3(in_shadow_normals);
	}

	*/
	color = vec4(post_rendering(uvs, final_color), 1.0);
}