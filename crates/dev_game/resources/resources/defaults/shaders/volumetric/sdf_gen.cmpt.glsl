#version 460 core
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;
layout(r16f, binding = 0) uniform image3D sdf_tex;
layout(location = 1) uniform vec3 points[1];
layout(location = 2) uniform float sphere_size;

void main() {
    // Get the pixel coord
    ivec3 pixel_coords = ivec3(gl_GlobalInvocationID.xyz);
    // Dafuq happened here with the terrain gen compute?
    // Calculate the SDF using the center points 
    float min_distance = 10000.0;
    for(int i = 0; i < points.length(); i++) {
        // Calculate the distance between the point and the pixel 
        float dist = distance(points[i], vec3(pixel_coords)) - 10;
        min_distance = min(min_distance, dist);
    }
    vec4 pixel = vec4(min_distance, 0, 0, 0);    
    
    // Write the pixel
    imageStore(sdf_tex, pixel_coords, pixel);
}