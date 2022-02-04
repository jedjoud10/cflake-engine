#version 460 core
layout(location = 0) in vec3 model_pos;
layout(location = 1) in vec3 model_normal;
layout(location = 2) in vec4 model_tangent;
layout(location = 3) in vec2 model_uv;
layout(location = 4) in vec3 model_color;
layout(location = 5) in uint material_type;
uniform mat4 mvp_matrix;
uniform mat4 model_matrix;
out vec3 m_normal;
out vec4 m_tangents;
out vec2 m_uv;
out vec3 m_position;
out vec3 m_color;
out flat uint m_material_type;

void main() {
	vec4 mvp_pos = mvp_matrix * vec4(model_pos, 1.0);
	vec3 model_matrix_pos = (model_matrix * vec4(model_pos, 1.0)).xyz;
	gl_Position = mvp_pos;

	// Pass the data to the next shader
	m_position = model_matrix_pos;
	m_normal = normalize((model_matrix * vec4(model_normal, 0.0)).xyz);
	m_color = model_color;
	m_material_type = material_type;
}