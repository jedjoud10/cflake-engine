// Create some volumetric fog!
#include "defaults\shaders\others\hashes.func.glsl"
// The result of the raymarching
struct VolumetricResult {
    vec3 color;
    float depth;
};
const int STEP_COUNT = 16;
// Sampling the SDF texture
float scene(vec3 point, sampler3D sdf_tex, float time) {
    vec3 scale = vec3(1, 1, 1) / 10;
    float d = texture(sdf_tex, -point * scale).x * (1/float(STEP_COUNT));
    return d;
}
float map(float x, float ra, float rb, float r2a, float r2b) {
    // https://stackoverflow.com/questions/3451553/value-remapping
    return r2a + (x - ra) * (r2b - r2a) / (rb - ra);
}
VolumetricResult volumetric(vec3 camera_position, vec2 uvs, vec3 pixel_forward, vec3 pixel_forward_projection, vec2 nf_planes, sampler3D sdf_tex, float time) {
    // Starting point at camera
    vec3 point = camera_position + pixel_forward;    
    float summed_d = scene(point, sdf_tex, time);
    vec3 last_point = point;
    for(int i = 0; i < STEP_COUNT; i++) { 
        summed_d += scene(point, sdf_tex, time);
        point += pixel_forward * 0.4;
    }
    // Offset
    summed_d += 0.8;
    // Cos
    float cos_a = dot(vec3(0, 0, -1), pixel_forward_projection);
    // Distance 
    float d_depth = distance(point, camera_position) * cos_a;
    d_depth = map(d_depth, nf_planes.x, nf_planes.y, 0, 1);
    // Return
    vec3 color = vec3(1, 1, 1);
    return VolumetricResult(color * summed_d * 0.2, 0.0001);   
    
}