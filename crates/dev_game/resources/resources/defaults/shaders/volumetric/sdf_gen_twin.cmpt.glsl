#version 460 core
#include "defaults\shaders\others\hashes.func.glsl"
layout(local_size_x = 4, local_size_y = 4, local_size_z = 4) in;
layout(r16f, binding = 0) uniform image3D sdf_tex;
layout(location = 1) uniform sampler3D sdf_tex_original;

float opSmoothUnion( float d1, float d2, float k ) {
    float h = clamp( 0.5 + 0.5*(d2-d1)/k, 0.0, 1.0 );
    return mix( d2, d1, h ) - k*h*(1.0-h); 
}
float opSmoothSubtraction( float d1, float d2, float k ) {
    float h = clamp( 0.5 - 0.5*(d2+d1)/k, 0.0, 1.0 );
    return mix( d2, -d1, h ) + k*h*(1.0-h); 
}

// Create some fBm noise from the SDF texture
float fBm(vec3 point) {
    float d = texture(sdf_tex_original, point).x - 0.3;
    for(int i = 1; i < 4; i++) {
        float l = pow(5.5354, i);
        float p = pow(0.2, i);
        // The inflated new distance
        float i_d = d - p;
        float new_d = texture(sdf_tex_original, point * l + hash31(i*24.2436)).x;
        // Clamp
        new_d = min(max(new_d, i_d), d);
        d = min(d, new_d);
    }
    return d;
}


void main() {
    // Get the pixel coord
    ivec3 pixel_coords = ivec3(gl_GlobalInvocationID.xyz);
    vec3 uvs = vec3(pixel_coords) / vec3(gl_NumWorkGroups.xyz * gl_WorkGroupSize.xyz);
    // Run the fBm stuff and save it to da texture
    float d = fBm(uvs * vec3(1, 1, 1));
    vec4 pixel = vec4(d, 0, 0, 0);
    // Write the pixel
    imageStore(sdf_tex, pixel_coords, pixel);
}