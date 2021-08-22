#version 460 core
layout(location = 0) in vec3 model_pos;
layout(location = 1) in vec3 model_normal;
layout(location = 2) in vec4 model_tangent;
layout(location = 3) in vec2 model_uv;
uniform mat4 vp_matrix;
out vec3 m_normal;
out vec4 m_tangents;
out vec2 m_uv;
out vec3 m_position;

void main() {
	vec4 vp_pos = vp_matrix * vec4(model_pos, 1.0);
	gl_Position = vp_pos;

	// Pass the data to the next shader
	m_position = vp_pos.xyz;
	m_normal = model_normal;	
	m_tangents = model_tangent;
	m_uv = model_uv;
}