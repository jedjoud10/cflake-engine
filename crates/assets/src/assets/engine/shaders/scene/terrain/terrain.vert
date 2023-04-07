#version 460 core

// Main attribute set vertex attributes
layout(location = 0) in vec4 position;

// Camera bind group buffer (creates a 'camera' object)
#include <engine/shaders/common/camera.glsl>
#include <engine/shaders/noises/noise3D.glsl>

// Push constants for the mesh matrix
layout(push_constant) uniform PushConstants {
    mat4 matrix;
} mesh;

// Data to give to the fragment shader
layout(location = 0) out vec3 m_position;
layout(location = 1) out vec3 m_normal;
layout(location = 2) out vec3 m_color;
layout(location = 3) out float effect;

void main() {
	// Model space -> World space -> Clip space
    vec4 world_pos = mesh.matrix * vec4(position.xyz, 1);
    vec4 projected = (camera.projection * camera.view) * world_pos; 
    gl_Position = projected;

    // Decode the normal data that was written in the W component of position
    uint part = floatBitsToUint(position.w);
    vec4 unpacked = unpackUnorm4x8(part);

    effect = ((snoise(world_pos.xyz * 2.3) * 0.5 + 0.5) * 0.4 + 0.7);

    // Set the output variables
    m_position = world_pos.xyz;
    m_normal = -(unpacked.xyz * 2 - 1.0);
    m_color = vec3(1);
    //m_color = unpacked.xyz;
}