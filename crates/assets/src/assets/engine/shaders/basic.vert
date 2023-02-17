#version 460 core

// Main attribute set vertex attributes
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec4 tangent;
layout(location = 3) in vec4 tex_coord;

// Camera bind group buffer (creates a 'camera' object)
#include <engine/shaders/common/camera.glsl>

// Push constants for the mesh matrix
layout(push_constant) uniform PushConstants {
    mat4 matrix;
} mesh;

// Data to give to the fragment shader
layout(location = 0) out vec3 m_position;
layout(location = 1) out vec3 m_normal;
layout(location = 2) out vec3 m_tangent;
layout(location = 3) out vec3 m_bitangent;
layout(location = 4) out vec2 m_tex_coord;

void main() {
    gl_Position = vec4(0);
    /*
	// Model space -> World space -> Clip space
    vec4 world_pos = mesh.matrix * vec4(position, 1);
    vec4 projected = (camera.projection * camera.view) * world_pos; 
    gl_Position = projected;

    // Set the output variables
    m_position = world_pos.xyz;
    m_normal = normalize((mesh.matrix * vec4(normal, 0)).xyz);
    m_tangent = normalize((mesh.matrix * vec4(tangent.xyz, 0)).xyz);
    m_tex_coord = tex_coord;

    // Calculate world space bitangent
	vec3 bitangent = cross(normalize(m_normal), normalize(tangent.xyz)) * tangent.w;
	m_bitangent = normalize((mesh.matrix * vec4(bitangent, 0.0)).xyz);
    */
}