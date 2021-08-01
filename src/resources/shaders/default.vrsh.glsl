#version 460 core
layout(location = 0) in vec3 model_pos;
layout(location = 1) in vec3 model_normal;
layout(location = 2) in vec3 model_tangent;
layout(location = 3) in vec2 model_uv;
uniform mat4 mvp_matrix;
uniform mat4 model_matrix;
out vec3 m_normal;
out vec3 m_position;

void main() {
	vec4 mvp_pos = mvp_matrix * vec4(model_pos, 1.0);
	vec4 model_matrix_pos = model_matrix * vec4(model_pos, 1.0);
	gl_Position = mvp_pos;
	m_position = model_matrix_pos.xyz;
	m_normal = normalize((model_matrix * vec4(model_normal, 0.0)).xyz);
}