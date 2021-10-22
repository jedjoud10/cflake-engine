#version 460 core
#includep {"0"}
// Load the color voxel function file
layout(local_size_x = 8, local_size_y = 8, local_size_z = 8) in;
layout(rgba8, binding = 0) uniform image3D color_voxel_image;
layout(rgba8, binding = 1) uniform image3D detail1_image;
layout(rgba32f, binding = 2) uniform image3D detail2_image;
layout(location = 1) uniform sampler3D voxel_sampler;
layout(location = 2) uniform vec3 node_pos;
layout(location = 3) uniform int node_size;
layout(location = 4) uniform int chunk_size;

void main() {
    // Get the pixel coord
    ivec3 pixel_coords = ivec3(gl_GlobalInvocationID.xyz);

    // Get the position
    vec3 pos = vec3(pixel_coords.xzy);    
    float size = float(node_size) / (float(chunk_size) - 2.0);
    pos *= size;
    pos += node_pos;        
    // Read the voxel data
    // Unpack the normal voxel
    vec4 voxel_pixel = texture(voxel_sampler, vec3(pixel_coords+1) / vec3(chunk_size, chunk_size, chunk_size)).rgba; 
    Voxel voxel = Voxel(unpack_density(ivec2(voxel_pixel.xy * 255)), int(voxel_pixel.z * 255), int(voxel_pixel.w * 255));      
    // Create the pixel value
    ColorVoxel color_voxel = get_color_voxel(pos, voxel_sampler, voxel, vec3(pixel_coords));
    vec4 pixel = vec4(color_voxel.color, 0.0);        
    // Write the pixel
    imageStore(color_voxel_image, pixel_coords, pixel);
}