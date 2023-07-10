#version 460 core

// G-Buffer data write
layout(location = 0) out vec4 gbuffer_albedo;
layout(location = 1) out vec4 gbuffer_normal;
layout(location = 2) out vec4 gbuffer_mask;

// Data given by the vertex shader
layout(location = 0) in vec3 m_position;
layout(location = 1) in vec3 m_local_position;
layout(location = 2) in flat uint draw; 

// Used to calculate barycentric coordinates
layout (constant_id = 1) const uint input_vertices_count = 1;
layout (constant_id = 2) const uint input_triangles_count = 1;
void main() {
	gbuffer_albedo = vec4(1);
	gbuffer_normal = vec4(0);
	gbuffer_mask = vec4(0);
}