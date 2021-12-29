#version 460 core
#load_defaults renderer
layout(location = 0) in vec3 model_pos;
layout(location = 1) in vec3 model_normal;
layout(location = 2) in vec4 model_tangent;
layout(location = 3) in vec2 model_uv;
layout(location = 4) in vec3 model_color;
uniform mat4 mvp_matrix;
uniform mat4 model_matrix;
out vec3 m_normal;
out vec4 m_tangents;
out vec2 m_uv;
out vec3 m_position;
out vec3 m_color;
out vec2 screen_space_pos;
out mat3 tbn;

void main() {
	vec4 mvp_pos = mvp_matrix * vec4(model_pos, 1.0);
	vec3 model_matrix_pos = (model_matrix * vec4(model_pos, 1.0)).xyz;
	screen_space_pos = mvp_pos.xy / mvp_pos.w;
	gl_Position = mvp_pos;

	// Pass the data to the next shader
	m_position = model_matrix_pos;
	m_normal = normalize((model_matrix * vec4(model_normal, 0.0)).xyz);
	vec3 bitangent = model_tangent.w * cross(model_tangent.xyz, model_normal);
	m_tangents = vec4(normalize((model_matrix * vec4(model_tangent.xyz, 0.0)).xyz), model_tangent.w);
	vec3 t = m_tangents.xyz;
	vec3 b = normalize((model_matrix * vec4(bitangent, 0.0)).xyz);
	vec3 n = m_normal;
	tbn = mat3(t, b, n);
	m_uv = model_uv;
	m_color = model_color;
}