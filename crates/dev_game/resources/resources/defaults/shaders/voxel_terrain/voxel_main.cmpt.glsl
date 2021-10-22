#version 460 core
#includep {"0"}
// Load the voxel function file
layout(local_size_x = 8, local_size_y = 8, local_size_z = 8) in;
layout(rgba8, binding = 0) uniform image3D voxel_image;
layout(location = 1) uniform vec3 node_pos;
layout(location = 2) uniform int node_size;
layout(location = 3) uniform int chunk_size;

void main() {
    // Get the pixel coord
    ivec3 pixel_coords = ivec3(gl_GlobalInvocationID.xyz);

    // Get the position
    vec3 pos = vec3(pixel_coords.xzy);    
    float size = float(node_size) / (float(chunk_size) - 2.0);
    pos *= size;
    pos += node_pos;              
    // Create the pixel value
    Voxel voxel = get_voxel(pos);
    ivec2 densities = pack_density(voxel.density);
    vec4 pixel = vec4(float(densities.x) / 255, float(densities.y) / 255, float(voxel.biomeID) / 255, float(voxel.materialID) / 255);    
    
    // Write the pixel
    imageStore(voxel_image, pixel_coords, pixel);
}