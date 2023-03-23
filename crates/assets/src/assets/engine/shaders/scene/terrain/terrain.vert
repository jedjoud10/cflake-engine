#version 460 core

// Main attribute set vertex attributes
layout(location = 0) in vec3 position;

// Camera bind group buffer (creates a 'camera' object)
#include <engine/shaders/common/camera.glsl>

// Push constants for the mesh matrix
layout(push_constant) uniform PushConstants {
    mat4 matrix;
} mesh;

// Data to give to the fragment shader
layout(location = 0) out vec3 m_position;

void main() {
	// Model space -> World space -> Clip space
    vec4 world_pos = mesh.matrix * vec4(position, 1);
    m_position = world_pos.xyz;
    vec4 projected = (camera.projection * camera.view) * world_pos; 
    gl_Position = projected;
    gl_PointSize = 8.0;
}