#version 460 core
out vec3 color;
uniform sampler2D diffuse_texture;
uniform sampler2D normals_texture;
uniform sampler2D position_texture;
uniform sampler2D emissive_texture;
uniform sampler2D depth_stencil_texture;

// Ambient sky gradient
uniform sampler2D default_sky_gradient;

uniform vec3 directional_light_dir;
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

vec3 czm_saturation(vec3 rgb, float adjustment)
{
    // Algorithm from Chapter 16 of OpenGL Shading Language
    const vec3 W = vec3(0.2125, 0.7154, 0.0721);
    vec3 intensity = vec3(dot(rgb, W));
    return mix(intensity, rgb, adjustment);
}

void main() {
	// Sample the textures
	vec3 normal = texture(normals_texture, uv_coordinates).xyz;
	vec3 diffuse = texture(diffuse_texture, uv_coordinates).xyz;
	vec3 position = texture(position_texture, uv_coordinates).xyz;
	vec3 emissive = texture(emissive_texture, uv_coordinates).xyz;

	// Calculate specular
	vec3 view_dir = normalize(view_pos - position);
	vec3 reflect_dir = reflect(-directional_light_dir, normal);
	float specular = pow(max(dot(view_dir, reflect_dir), 0), 64);
	
	// Calculate the diffuse lighting
	float light_val = max(dot(normal, normalize(directional_light_dir)), 0) * 2;

	// Used for ambient lighting
	float ambient_lighting_strengh = 0.2;
	float light_val_inverted = max(-dot(normal, normalize(directional_light_dir)), 0) * ambient_lighting_strengh;
	float sky_light_val = (dot(normal, vec3(0, 1, 0)) + 1) / 2.0; 
	vec3 ambient_lighting_color = czm_saturation(texture(default_sky_gradient, vec2(0, 1 - sky_light_val)).xyz, 3);

	// Add everything
	vec3 ambient_lighting = diffuse * ambient_lighting_color * ambient_lighting_strengh;
	vec3 final_color = ambient_lighting;
	final_color += light_val * diffuse;
	final_color + specular;

	vec3 depth_stencil = texture(depth_stencil_texture, uv_coordinates).xyz;
	depth_stencil.x = depth_stencil.x;
	vec3 depth = vec3(depth_stencil.x, 0, 0);

	if (debug_view == 0) {
		color = max(final_color, emissive);
		//color = ambient_lighting_color;
	} else if (debug_view == 1) {
		color = ambient_lighting;
	} else if (debug_view == 2) {
		color = diffuse;
	}
}