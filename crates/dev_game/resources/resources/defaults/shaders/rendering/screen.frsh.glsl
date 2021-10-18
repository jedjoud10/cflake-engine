#version 460 core
out vec3 color;
uniform sampler2D diffuse_texture;
uniform sampler2D normals_texture;
uniform sampler2D position_texture;
uniform sampler2D emissive_texture;

// Ambient sky gradient
uniform sampler2D default_sky_gradient;

uniform vec3 directional_light_dir;
uniform vec3 view_pos;
uniform int debug_view;
uniform ivec2 resolution;
uniform float time;
in vec2 uv_coordinates;

void main() {	
	// Sample the textures
	vec3 normal = normalize(texture(normals_texture, uv_coordinates).xyz);
	vec3 diffuse = texture(diffuse_texture, uv_coordinates).xyz;
	vec3 position = texture(position_texture, uv_coordinates).xyz;
	vec3 emissive = texture(emissive_texture, uv_coordinates).xyz;

	// Calculate specular
	vec3 view_dir = normalize(view_pos - position);
	vec3 reflect_dir = reflect(-directional_light_dir, normal);
	const float specular_strength = 0.0;
	float specular = pow(max(dot(view_dir, reflect_dir), 0), 32);
	
	// Calculate the diffuse lighting
	const float directional_light_strength = 2;
	float light_val = max(dot(normal, normalize(directional_light_dir)), 0) * directional_light_strength;

	// Used for ambient lighting
	float ambient_lighting_strength = 0.1;
	float light_val_inverted = max(-dot(normal, normalize(directional_light_dir)), 0);

	vec3 sky_normal_lookup = reflect(-view_dir, normal);
	float sky_light_val = dot(sky_normal_lookup, vec3(0, 1, 0)); 
	vec3 ambient_lighting_color = texture(default_sky_gradient, vec2(0, 1 - sky_light_val)).xyz;

	vec3 reflect_color = ambient_lighting_color;
	// Add everything
	vec3 ambient_lighting = diffuse * ambient_lighting_color * ambient_lighting_strength;
	vec3 final_color = ambient_lighting;
	final_color += diffuse * light_val;
	final_color += specular * specular_strength;
	if (debug_view == 0) {
		if (any(notEqual(emissive, vec3(0, 0, 0)))) {
			color = emissive;
		} else {
			color = final_color;
		}
		//color = ambient_lighting;
	} else if (debug_view == 1) {
		color = normal;
	} else if (debug_view == 2) {
		color = diffuse;
	} else if (debug_view == 3) {
		color = emissive;
		color = reflect_color;
	} else if (debug_view == 4) {
	}
}