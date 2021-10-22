#version 460 core
#includep {"0"}
// Load the voxel function file
layout(local_size_x = 8, local_size_y = 8, local_size_z = 8) in;
layout(r16, binding = 0) uniform image3D voxel_image;
layout(rgba8, binding = 1) uniform image3D material_image;
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
    Voxel voxel;
    MaterialVoxel material_voxel;
    get_voxel(pos, out voxel, out material_voxel);

    // Write the voxel pixel
    vec4 pixel = vec4(uint(clamp(voxel.density + 32767, 0, 65535)), 0, 0, 0);        
    // Write the material pixel
    imageStore(voxel_image, pixel_coords, pixel);
    vec4 material_pixel = vec4(material_voxel.material_id, material_voxel.biome_id, material_voxel.hardness, material_voxel.texture_id);
    imageStore(material_image, pixel_coords, material_pixel);
}