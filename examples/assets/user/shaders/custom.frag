#version 460 core

// G-Buffer data write
layout(location = 0) out vec4 gbuffer_albedo;
layout(location = 1) out vec4 gbuffer_normal;
layout(location = 2) out vec4 gbuffer_mask;

// Data given by the vertex shader
layout(location = 0) in vec3 m_position;
layout(location = 1) in vec3 m_normal;
layout(location = 2) in vec3 m_tangent;
layout(location = 3) in vec3 m_bitangent;
layout(location = 4) in vec2 m_tex_coord;

void main() {
	vec3 normal = normalize(m_normal);

	// Set the G-buffer values
	gbuffer_albedo = vec4(1);
	gbuffer_normal = vec4(normal, 0);
	gbuffer_mask = vec4(vec3(1, 1, 0), 0);
}