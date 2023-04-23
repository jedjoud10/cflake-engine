#version 460 core

// Vertex scaling factor. n / (n-3)
layout (constant_id = 0) const float scaling_factor = 1.0;

// Main attribute set vertex attributes
layout(location = 3) in vec2 packed;

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

uint cube_16( in vec3 nor )
{
    vec3 mor; uint  id;
                                    mor = nor.xyz; id = 0u;
    if( abs(nor.y) > abs(mor.x) ) { mor = nor.yzx; id = 1u; }
    if( abs(nor.z) > abs(mor.x) ) { mor = nor.zxy; id = 2u; }
    uint is = (mor.x<0.0)?1u:0u;
    vec2 uv = 0.5 + 0.5*mor.yz/abs(mor.x);
    uvec2 iuv = uvec2(round(uv*vec2(127.0,63.0)));
    return iuv.x | (iuv.y<<7u) | (id<<13u) | (is<<15u);
}
vec3 i_cube_16( uint data )
{
    uvec2 iuv = uvec2( data, data>>7u ) & uvec2(127u,63u);
    vec2 uv = vec2(iuv)*2.0/vec2(127.0,63.0) - 1.0;
    uint is = (data>>15u)&1u;
    vec3 nor = vec3((is==0u)?1.0:-1.0,uv.xy);
    uint id = (data>>13u)&3u;
         if(id==0u) nor = nor.xyz;
    else if(id==1u) nor = nor.zxy;
    else            nor = nor.yzx;
    return normalize(nor);
}

void main() {
    // Convert from 4 floats into uints
    uint packed_cell_position = floatBitsToUint(packed.x);
    uint packed_inner_position = floatBitsToUint(packed.y);

    // Positions only need 16 bits (1 byte for cell coord, 1 byte for inner vertex coord)
    vec4 cell_position = unpackUnorm4x8(packed_cell_position) * 255;
    vec4 inner_position = unpackSnorm4x8(packed_inner_position);
    vec4 position = cell_position + inner_position;

	// Model space -> World space -> Clip space
    vec4 world_pos = mesh.matrix * vec4(position.xyz * scaling_factor, 1);
    vec4 projected = (camera.projection * camera.view) * world_pos; 
    gl_Position = projected;

    // Set the output variables
    m_position = world_pos.xyz;
    //m_normal = -normals.xyz;
}