#version 460 core
out vec3 color;
uniform sampler2D color_texture;
uniform sampler2D normals_texture;
uniform sampler2D position_texture;
uniform vec3 view_pos;
uniform int debug_view;
in vec2 uv_coordinates;

void main() {
	vec3 normal = texture(normals_texture, uv_coordinates).xyz;
	vec3 diffuse = texture(color_texture, uv_coordinates).xyz;
	vec3 position = texture(position_texture, uv_coordinates).xyz;
	vec3 light_dir = vec3(0, 1, 0);
	vec3 view_dir = normalize(view_pos - position);
	vec3 reflect_dir = reflect(-light_dir, normal);
	float specular = pow(max(dot(view_dir, reflect_dir), 0), 64);
	float light_val = dot(normal, normalize(light_dir));
	vec3 final_color = vec3(specular, specular, specular) + light_val * diffuse;
	if (debug_view == 0) {
		color = final_color;
	} else if (debug_view == 1) {
		color = normal;
	} else if (debug_view == 2) {
		color = diffuse;
	}
}