// Create some volumetric fog!
#include "defaults\shaders\others\hashes.func.glsl"
// The result of the raymarching
struct VolumetricResult {
    vec3 color;
};
const int STEP_COUNT = 32;
const float MAX_DISTANCE = 1000;
// Sampling the SDF texture
float scene(vec3 point, sampler3D sdf_tex, float time) {
    vec3 scale = vec3(1, 1, 1) / 700;
    float d = texture(sdf_tex, -point * scale).x * 40.0;
    //d = max(point.y, d);
    d = max(abs(point.y)-5, d);
    
    return d;
}
VolumetricResult volumetric(vec3 camera_position, vec2 uvs, vec3 pixel_forward, vec3 pixel_forward_projection, vec2 nf_planes, sampler3D sdf_tex, float time) {
    // Starting point at camera
    vec3 point = camera_position + pixel_forward;    
    float summed_d = scene(point, sdf_tex, time);
    vec3 last_point = point;
    for(int i = 0; i < STEP_COUNT; i++) { 
        // Max distance
        if (distance(point, camera_position) > MAX_DISTANCE /*|| (point.y > 2 && pixel_forward.y > 0) || (point.y < 0.2 && pixel_forward.y < 0)*/) {
            return VolumetricResult(vec3(0, 0, 0));
        }
        // Offset the point using the forward vector and a dynamic step size
        summed_d += scene(point, sdf_tex, time);
        point += pixel_forward * 1.0;
    }
    // Return
    vec3 color = vec3(1, 1, 1);
    return VolumetricResult(color);   
    
}