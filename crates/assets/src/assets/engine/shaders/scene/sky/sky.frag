#version 460 core
layout(location = 0) out vec4 frag;

// Data given by the vertex shader
layout(location = 0) in vec3 m_position;

// Sky gradient texture map
layout(set = 1, binding = 0) uniform texture2D sky_map;
layout(set = 1, binding = 1) uniform sampler sky_map_sampler;

void main() {
	// Calculate sky color based on sun
	frag = vec4(normalize(m_position), 1.0);
}