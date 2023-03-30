#version 460 core

// Main attribute set vertex attributes
layout(location = 0) in vec4 position;

// Camera bind group buffer (creates a 'camera' object)
#include <engine/shaders/common/camera.glsl>

// Push constants for the mesh matrix
layout(push_constant) uniform PushConstants {
    mat4 matrix;
} mesh;

// Data to give to the fragment shader
layout(location = 0) out vec3 m_position;
layout(location = 1) out vec3 m_normal;

void main() {
	// Model space -> World space -> Clip space
    vec4 world_pos = mesh.matrix * vec4(position.xyz, 1);
    vec4 projected = (camera.projection * camera.view) * world_pos; 
    gl_Position = projected;

    // Decode the normal data that was written in the W component of position

    // Set the output variables
    m_position = world_pos.xyz;
    m_normal = vec3(0);
}