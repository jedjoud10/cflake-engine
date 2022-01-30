#version 460 core
#load general
out vec3 color;
uniform sampler2D diffuse_texture; // 0
uniform sampler2D emissive_texture; // 1
uniform sampler2D normals_texture; // 2
uniform sampler2D position_texture; // 3
uniform sampler2D depth_texture; // 4

// Ambient sky gradient
uniform sampler2D default_sky_gradient; // 5

uniform vec3 directional_light_dir;
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
	
	// Calculate the diffuse lighting
	const float directional_light_strength = 1.0;
	float light_val = max(dot(normal, normalize(directional_light_dir)), 0) * directional_light_strength;

	// Sky gradient texture moment
    vec3 pixel_dir = normalize((inverse(custom_vp_matrix) * vec4(uvs * 2 - 1, 0, 1)).xyz);
	float sky_uv_sampler = dot(pixel_dir, vec3(0, 1, 0));
	vec3 sky_color = texture(default_sky_gradient, vec2(0, sky_uv_sampler)).xyz;
	// Add the sun
	sky_color += max(pow(dot(pixel_dir, normalize(directional_light_dir)), 512), 0);

	// Pixel direction reflected by the surface normal
	vec3 reflected = reflect(pixel_dir, normal);
	vec3 reflected_color = texture(default_sky_gradient, vec2(0, reflected.y)).xyz;


	// Used for ambient lighting
	float ambient_lighting_strength = 0.02;
	float sky_light_val = dot(normal, vec3(0, 1, 0)); 
	vec3 ambient_lighting_color = texture(default_sky_gradient, vec2(0, sky_light_val)).xyz;
	
	// Add everything
	vec3 ambient_lighting = diffuse * ambient_lighting_color * ambient_lighting_strength;
	vec3 pixel_color = ambient_lighting;
	pixel_color += diffuse * light_val;
	
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

	// Depth test the sky
	if (depth > 0.999) {
		color = sky_color;
	} else {
		color = diffuse;
	}
}