#version 460 core
layout(location = 0) out vec4 frag;

// Data given by the vertex shader
layout(location = 0) in vec3 m_position;
layout(location = 1) in vec3 m_normal;
layout(location = 2) in vec3 m_tangent;
layout(location = 3) in vec3 m_bitangent;
layout(location = 4) in vec2 m_tex_coord;

// Camera, scene, and time shared objects
/*
//#include <engine/shaders/common/camera.glsl>
//#include <engine/shaders/common/scene.glsl>
//#include <engine/shaders/common/timing.glsl>

// Material scalar data
layout(set = 1, binding = 1) uniform MaterialData {
	vec3 tint;
	float roughness;
} material;

// Albedo / diffuse map
layout(set = 1, binding = 2) uniform texture2D albedo_map;
layout(set = 1, binding = 3) uniform sampler albedo_sampler;

// Normal map
layout(set = 1, binding = 4) uniform texture2D normal_map;
layout(set = 1, binding = 5) uniform sampler normal_sampler;
*/

void main() {
	frag = vec4(m_normal, 0);
}