#version 460 core
layout(location = 0) in vec3 mesh_pos;
layout(location = 1) in vec3 mesh_normal;
layout(location = 2) in vec4 mesh_tangent;
layout(location = 3) in vec2 mesh_uv;
layout(location = 4) in vec3 mesh_color;
uniform mat4 perspective_view_matrix;
uniform mat4 mesh_matrix;
out vec3 m_normal;
out vec3 m_tangent;
out vec3 m_bitangent;
out vec2 m_uv;
out vec3 m_position;
out vec3 m_color;

void main() {
	vec4 mesh_matrix_pos = (mesh_matrix * vec4(mesh_pos, 1.0));
	vec4 mvp_pos = perspective_view_matrix * mesh_matrix_pos;
	gl_Position = mvp_pos;

	// Pass the data to the next shader
	m_position = mesh_matrix_pos.xyz;
	m_normal = (mesh_matrix * vec4(mesh_normal, 0.0)).xyz;
	m_uv = mesh_uv;
	m_color = mesh_color;

	// Calculate the world tangent
	m_tangent = (mesh_matrix * vec4(mesh_tangent.xyz, 0.0)).xyz;
	float _sign = mesh_tangent.w;
	vec3 bitangent = cross(normalize(m_normal), normalize(mesh_tangent.xyz)) * _sign;
	m_bitangent = normalize((mesh_matrix * vec4(bitangent, 0.0)).xyz);
}