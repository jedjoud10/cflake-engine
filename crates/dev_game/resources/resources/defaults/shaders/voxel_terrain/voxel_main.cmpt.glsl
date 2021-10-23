#version 460 core
#includep {"0"}
// Load the voxel function file
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;
layout(binding = 0) writeonly uniform image3D voxel_image;
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
    /*
    Voxel voxel = Voxel(0.0);
    MaterialVoxel material_voxel = MaterialVoxel(0, 0, 0, 0);
    get_voxel(pos, voxel, material_voxel);
    */
    // Write the voxel pixel
    vec4 pixel = vec4(1.0, 0.0, 0.0, 1.0);        
    // Write the material pixel
    imageStore(voxel_image, pixel_coords, pixel);    
    /*
    vec4 material_pixel = vec4(material_voxel.material_id, material_voxel.biome_id, material_voxel.hardness, material_voxel.texture_id);
    imageStore(material_image, pixel_coords, material_pixel);    
    */
}