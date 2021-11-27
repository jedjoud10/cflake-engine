#version 460 core
out vec3 color;
uniform sampler2D diffuse_texture; // 0
uniform sampler2D normals_texture; // 1
uniform sampler2D position_texture; // 2
uniform sampler2D depth_texture; // 3

// Ambient sky gradient
uniform sampler2D default_sky_gradient; // 4
uniform sampler2D frame_stats; // 5

uniform vec3 directional_light_dir;
uniform mat4 custom_vp_matrix;
uniform vec2 nf_planes;
uniform int debug_view;
uniform vec3 camera_pos;
uniform ivec2 resolution;
uniform float time;
in vec2 uv_coordinates;
uniform float test;

void main() {
	vec3 diffuse = texture(diffuse_texture, uv_coordinates).xyz;
	color = diffuse;
	/*
	vec2 uvs = uv_coordinates;
	ivec2 pixel = ivec2(uv_coordinates * resolution);
	// Sample the textures
	vec3 normal = normalize(texture(normals_texture, uvs).xyz);
	vec3 diffuse = texture(diffuse_texture, uvs).xyz;
	vec3 position = texture(position_texture, uvs).xyz;
	
	// Calculate the diffuse lighting
	const float directional_light_strength = 1.0;
	float light_val = max(dot(normal, normalize(directional_light_dir)), 0) * directional_light_strength;

	// Used for ambient lighting
	float ambient_lighting_strength = 0.0;
	float light_val_inverted = max(-dot(normal, normalize(directional_light_dir)), 0);

	float sky_light_val = dot(normal, vec3(0, 1, 0)); 
	vec3 ambient_lighting_color = texture(default_sky_gradient, vec2(0, sky_light_val)).xyz;

	vec3 reflect_color = ambient_lighting_color;
	// Add everything
	vec3 ambient_lighting = diffuse * ambient_lighting_color * ambient_lighting_strength;
	vec3 final_color = ambient_lighting;
	final_color += diffuse * light_val;
	// If the diffuse color is above 1, that means that it is an emissive color instead
	if (all(greaterThan(diffuse, vec3(1, 1, 1)))) {
		final_color = diffuse;
	}

	float odepth = texture(depth_texture, uvs).x;
	float depth = (nf_planes.x * odepth) / (nf_planes.y - odepth * (nf_planes.y - nf_planes.x));

	// Sky gradient texture moment
    vec3 pixel_dir = normalize((inverse(custom_vp_matrix) * vec4(uvs * 2 - 1, 0, 1)).xyz);
	float sky_uv_sampler = dot(pixel_dir, vec3(0, 1, 0));
	vec3 sky_color = texture(default_sky_gradient, vec2(0, sky_uv_sampler)).xyz;
	// Add the sun
	sky_color += max(pow(dot(pixel_dir, normalize(directional_light_dir)), 512), 0);

	if (debug_view == 0) {
		// Depth test the sky
		if (depth > 0.999) {
			color = sky_color;
		} else {
			color = final_color;
		}
		// Show the frame stats at the top left corner of the screen
		if ((uvs.x * resolution.x < 400) && (uvs.y * resolution.y > resolution.y - 200)) {
			float x = (uvs.x * resolution.x) / 400;
			float y = (resolution.y - (uvs.y * resolution.y)) / 200;
			color = texture(frame_stats, vec2(x, y)).rgb;
		}
		color = vec3(1, 1, 1);
	} else if (debug_view == 1) {
		color = normal;
	} else if (debug_view == 2) {
		color = diffuse;
	} else if (debug_view == 3) {
		color = light_val * vec3(1, 1, 1);
	} else if (debug_view == 4) {
		color = ambient_lighting_color;
	}
	*/
}