#version 460 core
#include "defaults\shaders\others\hashes.func.glsl"
#include "defaults\shaders\noises\simplex.func.glsl"
#include "defaults\shaders\noises\voronoi.func.glsl"
#include "defaults\shaders\others\sdf.func.glsl"
#include "defaults\shaders\voxel_terrain\data.func.glsl"
// Load the voxel function file
layout(local_size_x = 8, local_size_y = 8, local_size_z = 8) in;
layout(binding = 0) readonly uniform image3D density_image;
layout(binding = 1) writeonly uniform image3D normals_image;
layout(binding = 2) writeonly uniform image3D materials_image;
layout(location = 2) uniform vec3 node_pos;
layout(location = 3) uniform int node_size;
layout(location = 4) uniform int chunk_size;

void main() {
    // Get the pixel coord#include_custom {"voxel_interpreter"}
    ivec3 pixel_coords = ivec3(gl_GlobalInvocationID.xyz);

    // Get the position
    vec3 pos = vec3(pixel_coords.xzy);    
    float size = float(node_size) / (float(chunk_size) - 2.0);
    pos *= size;
    pos += node_pos;             
}