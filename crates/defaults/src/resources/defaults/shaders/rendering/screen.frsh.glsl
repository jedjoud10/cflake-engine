#version 460 core
#load general
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
	float sky_x = (dot(directional_light_dir, vec3(0, 1, 0)) + 1) / 2.0;
	float sun_strength = clamp(sky_x * 6 - 2.4, 0, 1);
	
	// Calculate the diffuse lighting
	float light_val = max(dot(normal, normalize(directional_light_dir)), 0) * directional_light_strength * sun_strength;

	// Sky gradient texture moment
    vec3 pixel_dir = normalize((inverse(custom_vp_matrix) * vec4(uvs * 2 - 1, 0, 1)).xyz);
	float sky_uv_sampler = dot(pixel_dir, vec3(0, 1, 0));
	vec3 sky_color = texture(default_sky_gradient, vec2(sky_x, sky_uv_sampler)).xyz;
	// Add the sun
	sky_color += max(pow(dot(pixel_dir, normalize(directional_light_dir)), 512), 0) * sun_strength;

	// Pixel direction reflected by the surface normal
	vec3 reflected = reflect(pixel_dir, normal);
	vec3 reflected_color = texture(default_sky_gradient, vec2(sky_x, reflected.y)).xyz;

	// Used for ambient lighting
	float ambient_lighting_strength = 0.05;
	float sky_light_val = dot(normal, vec3(0, 1, 0)); 
	vec3 ambient_lighting_color = texture(default_sky_gradient, vec2(sky_x, sky_light_val)).xyz;
	
	// Add everything
	vec3 ambient_lighting = ambient_lighting_color * ambient_lighting_strength;
	vec3 pixel_color = ambient_lighting;
	pixel_color += diffuse * (light_val + 0.1);
	
	// Calculate some specular
	float specular_val = pow(clamp(dot(pixel_dir, reflect(directional_light_dir, normal)), 0, 1), 512);
	pixel_color += specular_val * 0.0;

	// Test emissive
	if (all(notEqual(emissive, vec3(0, 0, 0)))) {
		pixel_color = emissive;
	}

	// Calculate linear depth
	float odepth = texture(depth_texture, uvs).x;
	float depth = (nf_planes.x * odepth) / (nf_planes.y - odepth * (nf_planes.y - nf_planes.x));	

	// Shadow mapping calculations
	float shadow_mapped_depth = texture(shadow_map, uvs).r;

	// Depth test with the sky
	if (depth == 1.0) {
		color = sky_color;
	} else {
	}
	color = vec3(shadow_mapped_depth);
}