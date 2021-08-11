#version 460 core
out vec3 color;
uniform sampler2D diffuse_texture;
uniform sampler2D normals_texture;
uniform sampler2D position_texture;
uniform sampler2D emissive_texture;
uniform vec3 view_pos;
uniform int debug_view;
in vec2 uv_coordinates;

// Some tonemapping
vec3 aces(vec3 x) {
  const float a = 2.51;
  const float b = 0.03;
  const float c = 2.43;
  const float d = 0.59;
  const float e = 0.14;
  return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}

void main() {
	// Sample the textures
	vec3 normal = texture(normals_texture, uv_coordinates).xyz;
	vec3 diffuse = texture(diffuse_texture, uv_coordinates).xyz;
	vec3 position = texture(position_texture, uv_coordinates).xyz;
	vec3 emissive = texture(emissive_texture, uv_coordinates).xyz;

	// Light direction
	vec3 light_dir = vec3(0, 1, 0);

	// Calculate specular
	vec3 view_dir = normalize(view_pos - position);
	vec3 reflect_dir = reflect(-light_dir, normal);
	float specular = pow(max(dot(view_dir, reflect_dir), 0), 64);
	
	// Calculate the diffuse lighting
	float light_val = max(dot(normal, normalize(light_dir)), 0);

	// Used for ambient lighting
	vec3 ambient_lighting_color = vec3(0, 112, 204) / 255.0;
	float ambient_lighting_strengh = 0.1;
	float light_val_inverted = max(-dot(normal, normalize(light_dir)), 0) * ambient_lighting_strengh;

	// Add everything
	vec3 final_color = light_val_inverted * diffuse * ambient_lighting_color;
	final_color += light_val * diffuse;
	final_color + specular;

	if (debug_view == 0) {
		color = max(final_color, emissive);
		//color = aces(color);
	} else if (debug_view == 1) {
		color = normal;
	} else if (debug_view == 2) {
		color = diffuse;
	}
}