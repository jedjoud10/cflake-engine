#version 460 core
#include "defaults\shaders\others\hashes.func.glsl"
layout(local_size_x = 4, local_size_y = 4, local_size_z = 4) in;
layout(location = 0) uniform sampler3D sdf_tex;

float opSmoothUnion( float d1, float d2, float k ) {
    float h = clamp( 0.5 + 0.5*(d2-d1)/k, 0.0, 1.0 );
    return mix( d2, d1, h ) - k*h*(1.0-h); 
}
float opSmoothSubtraction( float d1, float d2, float k ) {
    float h = clamp( 0.5 - 0.5*(d2+d1)/k, 0.0, 1.0 );
    return mix( d2, -d1, h ) + k*h*(1.0-h); 
}

// Create some fBm noise from the SDF texture
float fBm(vec3 point, sampler3D sdf_tex, float time) {
    float d = texture(sdf_tex, point).x;
    for(int i = 0; i < 5; i++) {
        float l = pow(1.9, i);
        float p = pow(0.1, i);
        // The inflated new distance
        float i_d = d - p;
        float new_d = texture(sdf_tex, point * l + hash31(i)).x;
        // Clamp
        new_d = min(opSmoothSubtraction(-new_d, i_d, (sin(time) + 1) * 2), d);
        d = opSmoothUnion(d, new_d, 0.5);
    }
    return d;
}

void main() {
    // Get the pixel coord
    ivec3 pixel_coords = ivec3(gl_GlobalInvocationID.xyz);
    // Dafuq happened here with the terrain gen compute?
    // Get the cell coordinates
    // Run the fBm stuff and save it to da texture
    vec4 pixel = vec4(0, 0, 0, 0);
    // Write the pixel
    imageStore(sdf_tex, pixel_coords, pixel);
}