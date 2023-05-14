#version 460 core

// Vertex scaling factor. n / (n-3)
layout (constant_id = 0) const float scaling_factor = 1.0;

// Main attribute set vertex attributes
layout(location = 0) in vec4 packed;

// Camera bind group buffer (creates a 'camera' object)
#include <engine/shaders/common/camera.glsl>
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/math/packer.glsl>

// Data to give to the fragment shader
layout(location = 0) out vec3 m_position;
layout(location = 1) out vec3 m_local_position;
layout(location = 2) out vec3 m_normal;

// Contains position and scale value
layout(std430, set = 2, binding = 0) readonly buffer PositionScaleBuffer {
    vec4 data[];
} position_scale_buffer;

void main() {
    // Convert from 4 floats into uints 
    uint packed_cell_position = floatBitsToUint(packed.x);
    uint packed_inner_position = floatBitsToUint(packed.y);
    uint packed_normals = floatBitsToUint(packed.z);

    // Positions only need 16 bits (1 byte for cell coord, 1 byte for inner vertex coord)
    vec4 cell_position = unpackUnorm4x8(packed_cell_position) * 255;
    vec4 inner_position = unpackSnorm4x8(packed_inner_position);
    vec4 position = cell_position + inner_position;
    m_local_position = position.xyz;
    vec4 normals = unpackSnorm4x8(packed_normals);

	// Model space -> World space -> Clip space
    vec4 position_scale = position_scale_buffer.data[gl_DrawID];
    vec4 world_pos = vec4(((position.xyz * scaling_factor) * position_scale.w + position_scale.xyz), 1);
    vec4 projected = (camera.projection * camera.view) * world_pos; 
    gl_Position = projected;
    
    // Set the output variables
    m_position = world_pos.xyz;
    m_normal = -normals.xyz;
}