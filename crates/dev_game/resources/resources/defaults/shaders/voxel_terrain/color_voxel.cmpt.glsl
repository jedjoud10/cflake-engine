#version 460 core
#includep {"0"}
// Load the color voxel function file
layout(local_size_x = 8, local_size_y = 8, local_size_z = 8) in;
layout(binding = 0) writeonly uniform image3D color_image;
layout(location = 1) uniform sampler3D voxel_sampler;
layout(location = 2) uniform sampler3D material_sampler;
layout(location = 3) uniform vec3 node_pos;
layout(location = 4) uniform int node_size;
layout(location = 5) uniform int chunk_size;
layout(location = 6) uniform int depth;
void main() {
    // Get the pixel coord
    ivec3 pixel_coords = ivec3(gl_GlobalInvocationID.xyz);

    // Get the position
    vec3 pos = vec3(pixel_coords.xzy);    
    float size = float(node_size) / (float(chunk_size) - 2.0);
    pos *= size;
    pos += node_pos;        
    // Read the voxel data
    vec4 voxel_pixel = texture(voxel_sampler, vec3(pixel_coords+1) / vec3(chunk_size, chunk_size, chunk_size)).rgba; 
    Voxel voxel = Voxel((voxel_pixel.x * 65535) - 32767);   
    vec3 local_uv = vec3(pixel_coords+1) / vec3(chunk_size, chunk_size, chunk_size);
    vec4 mvp = texture(material_sampler, local_uv).rgba; 
    ColorVoxel color_voxel = ColorVoxel(vec3(0, 0, 0));
    get_color_voxel(pos, local_uv, voxel, MaterialVoxel(int(mvp.x * 255), int(mvp.y * 255), int(mvp.z * 255), int(mvp.w * 255)), depth, color_voxel);
    vec4 pixel = vec4(color_voxel.color, 0.0);     
    // Write the pixel
    imageStore(color_image, pixel_coords, pixel);
}