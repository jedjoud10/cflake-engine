#version 460 core
out vec3 color;
uniform sampler2D diffuse_texture;
uniform sampler2D normals_texture;
uniform sampler2D position_texture;
uniform sampler2D depth_texture;

// Ambient sky gradient
uniform sampler2D default_sky_gradient;

uniform sampler2D volumetric_texture;
uniform sampler2D volumetric_depth_texture;
uniform sampler3D sdf_texture;

uniform vec3 directional_light_dir;
uniform mat4 custom_vp_matrix;
uniform vec2 nf_planes;
uniform int debug_view;
uniform vec3 camera_pos;
uniform ivec2 resolution;
uniform float time;
in vec2 uv_coordinates;

void main() {	
	// Sample the textures
	vec2 uvs = uv_coordinates;
	ivec2 pixel = ivec2(uv_coordinates * resolution);
	vec3 normal = normalize(texture(normals_texture, uvs).xyz);
	vec3 diffuse = texture(diffuse_texture, uvs).xyz;
	vec3 position = texture(position_texture, uvs).xyz;
	
	// Calculate the diffuse lighting
	const float directional_light_strength = 1.0;
	float light_val = max(dot(normal, normalize(directional_light_dir)), 0) * directional_light_strength;

	// Used for ambient lighting
	float ambient_lighting_strength = 0.1;
	float light_val_inverted = max(-dot(normal, normalize(directional_light_dir)), 0);

	float sky_light_val = dot(normal, vec3(0, 1, 0)); 
	vec3 ambient_lighting_color = texture(default_sky_gradient, vec2(0, sky_light_val)).xyz;

	vec3 reflect_color = ambient_lighting_color;
	// Add everything
	vec3 ambient_lighting = diffuse * ambient_lighting_color * ambient_lighting_strength;
	vec3 final_color = ambient_lighting;
	final_color += diffuse * light_val;

	// Sample the volumetric result texture
	vec3 volumetric_color = texture(volumetric_texture, uvs).rgb;
	float new_depth = texture(volumetric_depth_texture, uvs).r;
	float depth = texture(depth_texture, uvs).x;
	float old_depth = (nf_planes.x * depth) / (nf_planes.y - depth * (nf_planes.y - nf_planes.x));
	// Compare the depths
	bool draw = old_depth > new_depth && new_depth != 0;

	// Sky gradient texture moment
    vec3 pixel_dir = normalize((inverse(custom_vp_matrix) * vec4(uvs * 2 - 1, 0, 1)).xyz);
	float sky_uv_sampler = dot(pixel_dir, vec3(0, 1, 0));
	vec3 sky_color = texture(default_sky_gradient, vec2(0, sky_uv_sampler)).xyz;
	// Add the sun
	sky_color += max(pow(dot(pixel_dir, normalize(directional_light_dir)), 512), 0);

	if (debug_view == 0) {
		// Depth test the sky
		if (old_depth > 0.999) {
			color = sky_color;
		} else {
			color = final_color;
		}
	} else if (debug_view == 1) {
		color = normal;
	} else if (debug_view == 2) {
		color = diffuse;
	} else if (debug_view == 3) {
		color = light_val * vec3(1, 1, 1);
	} else if (debug_view == 4) {
		color = ambient_lighting_color;
	}
}