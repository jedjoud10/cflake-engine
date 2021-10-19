#version 460 core
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;
layout(r8, binding = 0) uniform image3D sdf_tex;
uniform vec3 points[1];
uniform float sphere_size;

void main() {
    // Get the pixel coord
    ivec3 pixel_coords = ivec3(gl_GlobalInvocationID.xyz);

    // Calculate the SDF using the center points 
    float min_distance = 99999;
    for(int i = 0; i < (points.length()); i++) {
        // Calculate the distance between the point and the pixel 
        float dist = distance(points[i], vec3(pixel_coords)) - sphere_size;
        min_distance = min(min_distance, dist);
    }
    vec4 pixel = vec4(mod(float(pixel_coords.x), 3.0), 0, 0, 0);    
    
    // Write the pixel
    imageStore(sdf_tex, pixel_coords, pixel);
}