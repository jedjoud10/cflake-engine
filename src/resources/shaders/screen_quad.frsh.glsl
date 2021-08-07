#version 460 core
out vec3 color;
uniform sampler2D color_texture;
uniform sampler2D normals_texture;

in vec2 uv_coordinates;

void main() {
	vec3 normals = texture(normals_texture, uv_coordinates).xyz;
	float val = dot(normals, vec3(0.0, 1.0, 0.0));
	color = texture(normals_texture, uv_coordinates).xyz;
	color = vec3(val, val, val);
}