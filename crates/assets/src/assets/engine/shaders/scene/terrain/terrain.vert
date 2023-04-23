#version 460 core

// Vertex scaling factor. n / (n-3)
layout (constant_id = 0) const float scaling_factor = 1.0;

// Main attribute set vertex attributes
layout(location = 3) in vec2 packed;

// Camera bind group buffer (creates a 'camera' object)
#include <engine/shaders/common/camera.glsl>
#include <engine/shaders/noises/noise3D.glsl>
#include <engine/shaders/math/packer.glsl>

// Push constants for the mesh matrix
layout(push_constant) uniform PushConstants {
    mat4 matrix;
} mesh;

// Data to give to the fragment shader
layout(location = 0) out vec3 m_position;
layout(location = 1) out vec3 m_normal;

void main() {
    // Convert from 4 floats into uints 
    uint packed_cell_position_packed_normals = floatBitsToUint(packed.x);
    uint packed_inner_position_packed_normals = floatBitsToUint(packed.y);

    uint packed_normals_first = (packed_cell_position_packed_normals >> 24) << 8;
    uint packed_normals_second = packed_inner_position_packed_normals >> 24;
    uint combined_packed_normals = packed_normals_first | packed_normals_second;
    vec3 normals = i_cube_16(combined_packed_normals);

    // Positions only need 16 bits (1 byte for cell coord, 1 byte for inner vertex coord)
    vec4 cell_position = unpackUnorm4x8(packed_cell_position_packed_normals) * 255;
    vec4 inner_position = unpackSnorm4x8(packed_inner_position_packed_normals);
    vec4 position = cell_position + inner_position;

	// Model space -> World space -> Clip space
    vec4 world_pos = mesh.matrix * vec4(position.xyz * scaling_factor, 1);
    vec4 projected = (camera.projection * camera.view) * world_pos; 
    gl_Position = projected;

    // Set the output variables
    m_position = world_pos.xyz;
    m_normal = -normals.xyz;
}